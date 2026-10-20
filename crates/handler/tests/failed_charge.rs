//! The record a failed regular charge leaves, as the mainnet handler sees it.
//!
//! A frame's result carries its gas to the handler, and with it the regular gas the frame had
//! left before a regular charge that failed ([`Gas::failed_charge_remaining`]). The handler here is
//! the mainnet one, observed through an override that leaves its behaviour alone: each frame's
//! result is recorded as the frame returns, before its caller settles it. One test calls
//! `return_create` directly instead, for the result it hands to the handler.

use bytecode::opcode::{CALL, POP, PUSH0, PUSH1, PUSH20, STOP};
use context::{
    result::{EVMError, ExecutionResult},
    Context, ContextSetters, Evm, TxEnv,
};
use context_interface::{ContextTr, Database, JournalTr};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{interpreter_action::FrameInit, Gas, InstructionResult, InterpreterResult};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{
    instructions::EthInstructions, return_create, EthPrecompiles, EvmTr, FrameResult, Handler,
    ItemOrResult, MainContext, MainnetContext, MainnetEvm,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<DbError>;

const SPEC: SpecId = SpecId::OSAKA;
/// Contract the transaction calls.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// Contract [`CONTRACT`] calls.
const CHILD: Address = address!("0x000000000000000000000000000000000000c41d");

/// The mainnet handler, recording each frame's result and failed-charge record as the frame
/// returns.
#[derive(Default)]
struct Observer {
    returned: Vec<(InstructionResult, Option<u64>)>,
}

impl Handler for Observer {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = context::result::HaltReason;

    /// The default loop, recording each frame result before it goes back to its caller.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        let mut record = |result: &FrameResult| {
            self.returned.push((
                result.instruction_result(),
                result.gas().failed_charge_remaining(),
            ));
        };
        if let ItemOrResult::Result(result) = evm.frame_init(first_frame_input)? {
            record(&result);
            return Ok(result);
        }
        loop {
            let result = match evm.frame_run()? {
                ItemOrResult::Item(init) => match evm.frame_init(init)? {
                    ItemOrResult::Item(_) => continue,
                    ItemOrResult::Result(result) => result,
                },
                ItemOrResult::Result(result) => result,
            };
            record(&result);
            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }
}

/// Runs a call to [`CONTRACT`], which calls [`CHILD`] with `child_gas` and stops, and returns the
/// transaction's result and every frame's result and record, innermost first.
fn run(child_gas: u8, child: &[u8]) -> (ExecutionResult, Vec<(InstructionResult, Option<u64>)>) {
    // CALL(child_gas, CHILD, 0, 0, 0, 0, 0), POP, STOP
    let mut contract = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20];
    contract.extend_from_slice(CHILD.as_slice());
    contract.extend_from_slice(&[PUSH1, child_gas, CALL, POP, STOP]);

    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    for (address, code) in [(CONTRACT, contract), (CHILD, child.to_vec())] {
        let code = Bytecode::new_raw(Bytes::from(code));
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    let ctx: TestContext = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SPEC));
    let mut evm: TestEvm = Evm::new(
        ctx,
        EthInstructions::new_mainnet_with_spec(SPEC),
        EthPrecompiles::new(SPEC),
    );
    evm.ctx.set_tx(
        TxEnv::builder_for_bench()
            .kind(TxKind::Call(CONTRACT))
            .gas_price(0)
            .gas_limit(100_000)
            .build_fill(),
    );

    let mut observer = Observer::default();
    let result = observer.run(&mut evm).expect("transaction runs");
    (result, observer.returned)
}

/// A child that halts on a failed static charge hands the handler the regular gas it had left
/// before the charge, although the halt zeroed its gas. Its caller, whose charges all succeeded,
/// carries no record: it does not take over its child's.
#[test]
fn test_a_halted_child_hands_its_record_to_the_handler() {
    // The child's 5: two `PUSH0`s (4) leave 1 against the third's 2.
    let (result, returned) = run(5, &[PUSH0, PUSH0, PUSH0, STOP]);

    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        returned,
        [
            (InstructionResult::OutOfGas, Some(1)),
            (InstructionResult::Stop, None),
        ]
    );
}

/// A child whose charges all succeed hands the handler no record.
#[test]
fn test_a_child_that_stops_hands_no_record() {
    let (result, returned) = run(5, &[PUSH0, PUSH0, STOP]);

    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        returned,
        [
            (InstructionResult::Stop, None),
            (InstructionResult::Stop, None),
        ]
    );
}

/// `return_create`'s own regular charges record what the frame had when they fail: the code
/// deposit's from Homestead (200 to deposit one byte, here against 100), and the code hash's
/// under EIP-8037 (6, here against 5). Before Homestead a deposit the frame cannot pay deploys
/// empty code, and the record is left on a creation that returns successfully.
#[test]
fn test_return_create_records_its_failed_charges() {
    for (spec, gas_limit, expected) in [
        (SpecId::FRONTIER, 100, InstructionResult::Return),
        (SpecId::HOMESTEAD, 100, InstructionResult::OutOfGas),
        (SpecId::AMSTERDAM, 5, InstructionResult::OutOfGas),
    ] {
        let mut ctx: TestContext = Context::mainnet()
            .with_db(Db::default())
            .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(spec));
        ctx.journal_mut().load_account(CONTRACT).unwrap();
        let checkpoint = ctx.journal_mut().checkpoint();
        let mut result = InterpreterResult::new(
            InstructionResult::Return,
            Bytes::from_static(&[0]),
            Gas::new(gas_limit),
        );

        return_create(&mut ctx, checkpoint, &mut result, CONTRACT);

        assert_eq!(result.result, expected, "{spec:?}");
        assert_eq!(result.gas.remaining(), gas_limit, "{spec:?}: nothing spent");
        assert_eq!(
            result.gas.failed_charge_remaining(),
            Some(gas_limit),
            "{spec:?}"
        );
        assert_eq!(result.gas.withheld_crossing(), None, "{spec:?}");
    }
}
