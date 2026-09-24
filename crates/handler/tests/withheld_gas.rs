//! Withheld regular gas across a frame boundary, through the mainnet handler.
//!
//! The interpreter here gets one extra opcode, `WITHHOLD`, that does what a consumer capping a
//! frame's regular gas does: it moves the popped amount from the frame's spendable gas to its
//! withheld gas ([`Gas::withhold`]). Every test runs a transaction twice, once withholding and once
//! withholding nothing at the same cost, and compares what the caller of the withholding frame
//! sees: withheld gas is regular gas held back from regular charges, not gas spent, so it goes
//! back to the caller wherever unspent gas does.
//!
//! The handler is the mainnet one, observed through an override that leaves its behaviour alone:
//! each frame's gas is recorded as the frame returns, before its caller settles it.
//!
//! [`Gas::withhold`]: interpreter::Gas::withhold

use bytecode::opcode::{
    CALL, GAS, INVALID, MLOAD, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH4, RETURN, REVERT, STOP,
};
use context::{
    result::{EVMError, ExecutionResult, HaltReason, Output},
    Context, ContextSetters, Evm, TxEnv,
};
use context_interface::Database;
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, GasTracker, Instruction,
    InstructionContext, InstructionExecResult, InstructionResult,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{
    instructions::EthInstructions, EthPrecompiles, EvmTr, FrameResult, Handler, ItemOrResult,
    MainContext, MainnetContext, MainnetEvm,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<DbError>;

/// Contract the transaction calls.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// Contract [`CONTRACT`] calls.
const CHILD: Address = address!("0x000000000000000000000000000000000000c41d");

/// Withholds the popped amount of the frame's regular gas.
const WITHHOLD: u8 = 0x0c;

/// Regular gas budget of a transaction; the gas limit above it is the reservoir.
const REGULAR_CAP: u64 = 2_000_000;
/// The reservoir a call transaction starts with.
const RESERVOIR: u64 = 100_000;
/// Gas [`CONTRACT`] forwards to [`CHILD`] with an explicit gas argument.
const CHILD_GAS: u64 = 500_000;
/// Gas the withholding frame withholds.
const WITHHELD: u64 = 400_000;

fn withhold(context: InstructionContext<'_, TestContext, EthInterpreter>) -> InstructionExecResult {
    let amount = context.interpreter.stack.pop()?.saturating_to();
    context.interpreter.gas.withhold(amount);
    Ok(())
}

/// Bytecode under construction.
#[derive(Clone, Default)]
struct Code(Vec<u8>);

impl Code {
    fn op(mut self, op: u8) -> Self {
        self.0.push(op);
        self
    }

    /// Pushes `value` as a four-byte word, so the gas the code costs does not depend on it.
    fn push(mut self, value: u64) -> Self {
        let value = u32::try_from(value).expect("test amounts fit in four bytes");
        self.0.push(PUSH4);
        self.0.extend_from_slice(&value.to_be_bytes());
        self
    }

    fn withhold(self, amount: u64) -> Self {
        self.push(amount).op(WITHHOLD)
    }

    /// `CALL(gas, CHILD, 0, 0, 0, 0, 0)`, dropping the success flag.
    fn call_child(mut self, gas: u64) -> Self {
        self = self.op(PUSH0).op(PUSH0).op(PUSH0).op(PUSH0).op(PUSH0);
        self.0.push(PUSH20);
        self.0.extend_from_slice(CHILD.as_slice());
        self.push(gas).op(CALL).op(POP)
    }

    /// Returns what `GAS` reads at this point, as a 32-byte word.
    fn return_gas(self) -> Self {
        self.op(GAS)
            .op(PUSH0)
            .op(MSTORE)
            .op(PUSH1)
            .op(32)
            .op(PUSH0)
            .op(RETURN)
    }

    fn revert(self) -> Self {
        self.op(PUSH0).op(PUSH0).op(REVERT)
    }

    fn build(self) -> Bytecode {
        Bytecode::new_raw(Bytes::from(self.0))
    }
}

/// The mainnet handler, recording each frame's gas as the frame returns.
#[derive(Default)]
struct Observer {
    returned: Vec<(InstructionResult, GasTracker)>,
}

impl Handler for Observer {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = HaltReason;

    /// The default loop, recording each frame result before it goes back to its caller.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        let mut record = |result: &FrameResult| {
            self.returned
                .push((result.instruction_result(), *result.gas().tracker()));
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

/// What the observed handler saw of one transaction.
struct Run {
    result: ExecutionResult,
    /// Every frame's result and gas as it returned, innermost first.
    returned: Vec<(InstructionResult, GasTracker)>,
}

impl Run {
    /// What [`Code::return_gas`] read in [`CONTRACT`]: its remaining gas after the call returned.
    fn contract_gas(&self) -> u64 {
        let ExecutionResult::Success {
            output: Output::Call(output),
            ..
        } = &self.result
        else {
            panic!("the contract returns, got {:?}", self.result);
        };
        U256::from_be_slice(output).saturating_to()
    }

    /// [`CHILD`]'s result and gas as it returned.
    fn child(&self) -> (InstructionResult, GasTracker) {
        assert_eq!(self.returned.len(), 2, "one child and the top frame");
        self.returned[0]
    }
}

/// Runs a call to [`CONTRACT`], holding `contract`, with [`CHILD`] holding `child`, on Amsterdam
/// with a reservoir.
fn run(contract: Code, child: Code) -> Run {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    for (address, code) in [(CONTRACT, contract), (CHILD, child)] {
        let code = code.build();
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    let ctx: TestContext = Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
        cfg.tx_gas_limit_cap = Some(REGULAR_CAP);
    });

    let mut instructions = EthInstructions::new_mainnet_with_spec(SpecId::AMSTERDAM);
    instructions.insert_instruction(WITHHOLD, Instruction::new(withhold), 0);
    let mut evm: TestEvm = Evm::new(ctx, instructions, EthPrecompiles::new(SpecId::AMSTERDAM));
    evm.ctx.set_tx(
        TxEnv::builder_for_bench()
            .kind(TxKind::Call(CONTRACT))
            .gas_price(0)
            .gas_limit(REGULAR_CAP + RESERVOIR)
            .build_fill(),
    );

    let mut observer = Observer::default();
    let result = observer.run(&mut evm).expect("transaction runs");
    Run {
        result,
        returned: observer.returned,
    }
}

/// Runs [`CONTRACT`] calling a [`CHILD`] that withholds `withheld` and then runs `rest`.
fn child_withholds(withheld: u64, rest: Code) -> Run {
    let contract = Code::default().call_child(CHILD_GAS).return_gas();
    let mut child = Code::default().withhold(withheld);
    child.0.extend_from_slice(&rest.0);
    run(contract, child)
}

/// A child that withholds gas and then stops, reverts or halts returns to its caller what it
/// would have returned with nothing withheld.
#[test]
fn test_a_child_returns_its_withheld_gas_like_unspent_gas() {
    let stop = Code::default().op(PUSH0).op(PUSH0).op(MSTORE).op(STOP);
    let revert = Code::default().revert();
    let halt = Code::default().op(INVALID);

    for (rest, expected) in [
        (stop, InstructionResult::Stop),
        (revert, InstructionResult::Revert),
        (halt, InstructionResult::InvalidFEOpcode),
    ] {
        let plain = child_withholds(0, rest.clone());
        let withheld = child_withholds(WITHHELD, rest);

        let (result, gas) = withheld.child();
        assert_eq!(result, expected);
        assert_eq!(plain.child().0, expected);
        // The frame result carries the withheld gas; the caller's settle spends it on a halt,
        // as it spends the rest of a halting child's gas.
        assert_eq!(gas.withheld(), WITHHELD);
        assert_eq!(gas.remaining(), plain.child().1.remaining());
        assert_eq!(
            withheld.contract_gas(),
            plain.contract_gas(),
            "the caller's remaining gas after a {expected:?}"
        );
        assert_eq!(
            withheld.result.gas().total_gas_spent(),
            plain.result.gas().total_gas_spent()
        );
        assert_eq!(gas.withheld_crossing(), None);
    }
}

/// A child whose regular charge fails within its withheld gas halts, like any out-of-gas, and
/// its frame result carries the crossing to the handler. Its caller loses the forward, exactly
/// as it does when the child halts with nothing withheld.
#[test]
fn test_a_crossing_halt_reaches_the_handler() {
    // One word of memory at an offset whose expansion costs more than the child keeps spendable
    // but less than its total.
    let expand = Code::default().push(200_000).op(MLOAD).op(STOP);

    let crossed = child_withholds(CHILD_GAS - 10_000, expand.clone());
    let unwithheld = child_withholds(0, expand);
    let halted = child_withholds(CHILD_GAS - 10_000, Code::default().op(INVALID));

    let (result, gas) = crossed.child();
    assert_eq!(result, InstructionResult::MemoryOOG);
    let crossing = gas
        .withheld_crossing()
        .expect("the crossing reaches the handler");
    assert!(crossing.spendable() < crossing.cost());
    assert!(crossing.cost() - crossing.spendable() <= CHILD_GAS - 10_000);
    assert_eq!(
        unwithheld.child().0,
        InstructionResult::Stop,
        "the total pays it"
    );

    assert_eq!(crossed.contract_gas(), halted.contract_gas());
    // The caller does not take over the child's record.
    assert_eq!(crossed.returned[1].1.withheld_crossing(), None);
}

/// Gas a caller withholds is forwarded like the rest: the child's gas limit is 63/64 of the
/// caller's total, the child starts with nothing withheld, and the caller's gas after the call is
/// what it is with nothing withheld.
#[test]
fn test_a_caller_forwards_its_withheld_gas() {
    let all_gas = u64::from(u32::MAX);
    let run_with = |withheld| {
        run(
            Code::default()
                .withhold(withheld)
                .call_child(all_gas)
                .return_gas(),
            Code::default().op(STOP),
        )
    };

    let plain = run_with(0);
    let withheld = run_with(REGULAR_CAP / 2);

    let (_, child) = withheld.child();
    assert_eq!(child.limit(), plain.child().1.limit());
    assert!(child.limit() > REGULAR_CAP / 2);
    assert_eq!(child.withheld(), 0);
    assert_eq!(withheld.contract_gas(), plain.contract_gas());
}
