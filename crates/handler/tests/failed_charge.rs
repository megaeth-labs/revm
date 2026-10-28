//! The record a failed charge leaves, as the mainnet handler sees it.
//!
//! A frame's result carries its gas to the handler, and with it the regular gas the frame had
//! left before a failed charge that ended it with an out-of-gas
//! ([`Gas::failed_charge_remaining`]). The handler here is the mainnet one, observed through an
//! override that leaves its behaviour alone: each frame's result is recorded as the frame
//! returns, before its caller settles it. The `return_create` tests call it directly instead,
//! for the result it hands to the handler, and the precompile tests call
//! `precompile_output_to_interpreter_result` directly as well.

use bytecode::opcode::{
    BALANCE, CALL, POP, PUSH0, PUSH1, PUSH20, PUSH4, SELFDESTRUCT, SLOAD, SSTORE, STOP,
};
use context::{
    result::{EVMError, ExecutionResult},
    Context, ContextSetters, Evm, TxEnv,
};
use context_interface::{
    cfg::{GasId, GasParams},
    ContextTr, Database, JournalTr,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    instructions::gas_table_spec, interpreter_action::FrameInit, Gas, InstructionResult,
    InterpreterResult, SStoreResult,
};
use precompile::{PrecompileHalt, PrecompileOutput};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{
    instructions::EthInstructions, precompile_output_to_interpreter_result, return_create,
    EthPrecompiles, EvmTr, FrameResult, Handler, ItemOrResult, MainContext, MainnetContext,
    MainnetEvm,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<DbError>;

/// A frame's result and failed-charge record, as it returned.
type Returned = (InstructionResult, Option<u64>);

const SPEC: SpecId = SpecId::OSAKA;
/// Regular gas budget of a transaction; the gas limit above it is the reservoir.
const REGULAR_CAP: u64 = 1_000_000;
/// Contract the transaction calls.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// Contract [`CONTRACT`] calls.
const CHILD: Address = address!("0x000000000000000000000000000000000000c41d");
/// An account no frame has touched, so its first access is cold.
const COLD: Address = address!("0x00000000000000000000000000000000000c01d0");
/// An account delegated to [`COLD`] (EIP-7702).
const DELEGATED: Address = address!("0x0000000000000000000000000000000000de1e9a");
/// The `ECRECOVER` precompile, which costs 3,000.
const ECRECOVER: Address = address!("0x0000000000000000000000000000000000000001");
/// The `IDENTITY` precompile, which costs 15 for no input.
const IDENTITY: Address = address!("0x0000000000000000000000000000000000000004");

/// The mainnet handler, recording each frame's result and failed-charge record as the frame
/// returns.
#[derive(Default)]
struct Observer {
    returned: Vec<Returned>,
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
                ItemOrResult::Result(mut result) => {
                    evm.frame_run_result(&mut result)?;
                    result
                }
            };
            record(&result);
            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }
}

/// `CALL(gas, to, 0, 0, 0, 0, 0)`, dropping the success flag. The pushes cost 21; `PUSH1 0`
/// rather than `PUSH0`, so the code runs on every spec.
fn call(to: Address, gas: u32) -> Vec<u8> {
    let mut code = [PUSH1, 0].repeat(5);
    code.push(PUSH20);
    code.extend_from_slice(to.as_slice());
    code.push(PUSH4);
    code.extend_from_slice(&gas.to_be_bytes());
    code.extend_from_slice(&[CALL, POP]);
    code
}

/// `PUSH20 address`.
fn push(address: Address) -> Vec<u8> {
    let mut code = vec![PUSH20];
    code.extend_from_slice(address.as_slice());
    code
}

/// A transaction in which [`CONTRACT`] calls an account and stops.
struct Run {
    spec: SpecId,
    reservoir: u64,
    prices: Vec<(GasId, u64)>,
    accounts: Vec<(Address, Bytecode)>,
}

impl Run {
    const fn on(spec: SpecId) -> Self {
        Self {
            spec,
            reservoir: 0,
            prices: Vec::new(),
            accounts: Vec::new(),
        }
    }

    /// The reservoir the transaction starts with, which every frame inherits (EIP-8037).
    const fn reservoir(mut self, reservoir: u64) -> Self {
        self.reservoir = reservoir;
        self
    }

    /// Prices `id` at `value` in the schedule.
    fn price(mut self, id: GasId, value: u64) -> Self {
        self.prices.push((id, value));
        self
    }

    fn account(mut self, address: Address, code: Bytecode) -> Self {
        self.accounts.push((address, code));
        self
    }

    /// Runs [`CONTRACT`] calling `to` with `gas`, with [`CHILD`] holding `child`, and returns the
    /// transaction's result and every frame's result and record, innermost first.
    fn call(self, to: Address, gas: u32, child: &[u8]) -> (ExecutionResult, Vec<Returned>) {
        let mut contract = call(to, gas);
        contract.push(STOP);

        let mut db = Db::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo::from_balance(U256::from(10u128.pow(21))),
        );
        let raw = |code: Vec<u8>| Bytecode::new_raw(Bytes::from(code));
        let accounts = [(CONTRACT, raw(contract)), (CHILD, raw(child.to_vec()))];
        for (address, code) in accounts.into_iter().chain(self.accounts) {
            db.insert_account_info(
                address,
                AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
            );
        }
        let spec = self.spec;
        let prices = self.prices;
        let ctx: TestContext = Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(spec);
            cfg.tx_gas_limit_cap = Some(REGULAR_CAP);
            cfg.gas_params.override_gas(prices);
        });
        let mut evm: TestEvm = Evm::new(
            ctx,
            EthInstructions::new_mainnet_with_spec(spec),
            EthPrecompiles::new(spec),
        );
        evm.ctx.set_tx(
            TxEnv::builder_for_bench()
                .kind(TxKind::Call(CONTRACT))
                .gas_price(0)
                .gas_limit(REGULAR_CAP + self.reservoir)
                .build_fill(),
        );

        let mut observer = Observer::default();
        let result = observer.run(&mut evm).expect("transaction runs");
        (result, observer.returned)
    }
}

/// Runs [`CONTRACT`] calling [`CHILD`], holding `child`, with `child_gas` on [`SPEC`].
fn run(child_gas: u32, child: &[u8]) -> (ExecutionResult, Vec<Returned>) {
    Run::on(SPEC).call(CHILD, child_gas, child)
}

/// The static gas of `opcode` on [`SPEC`].
fn static_gas(opcode: u8) -> u64 {
    gas_table_spec(SPEC)[opcode as usize].into()
}

/// Asserts that the frame [`CONTRACT`] called halted `result` with `record`, and that
/// [`CONTRACT`] stopped with none.
#[track_caller]
fn assert_called_frame(
    (result, returned): (ExecutionResult, Vec<Returned>),
    expected: InstructionResult,
    record: u64,
) {
    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        returned,
        [(expected, Some(record)), (InstructionResult::Stop, None)]
    );
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

/// An `SSTORE` that creates a slot, under EIP-8037, charges the slot's state gas after its
/// regular charges. A charge the reservoir cannot cover spills onto regular gas; when the frame
/// cannot pay the spill either, the frame halts out of gas, and the record holds the regular gas
/// it had before the state charge. The reservoir pays nothing, or part of the charge: either way
/// the record is regular gas alone.
#[test]
fn test_a_failed_state_charge_records_the_regular_gas_left() {
    let params = GasParams::new_spec(SpecId::AMSTERDAM);
    let new_slot = SStoreResult {
        original_value: U256::ZERO,
        present_value: U256::ZERO,
        new_value: U256::from(1),
    };
    // `PUSH1` and `PUSH0`, then `SSTORE`'s regular charges on a cold, new slot.
    let regular = 5
        + u64::from(gas_table_spec(SpecId::AMSTERDAM)[SSTORE as usize])
        + params.sstore_static_gas()
        + params.sstore_dynamic_gas(true, &new_slot, true);
    let state = params.get(GasId::sstore_set_state_gas());
    let child_gas = 30_000;
    let left = child_gas - regular;

    for reservoir in [0, state / 2] {
        assert!(left < state - reservoir, "the spill cannot be paid");
        let outcome = Run::on(SpecId::AMSTERDAM).reservoir(reservoir).call(
            CHILD,
            child_gas as u32,
            &[PUSH1, 1, PUSH0, SSTORE, STOP],
        );

        assert_called_frame(outcome, InstructionResult::OutOfGas, left);
    }
}

/// `return_create`'s state and history charges for the deposited code draw the reservoir first
/// and spill onto regular gas; a spill the frame cannot pay is `return_create`'s own out-of-gas,
/// and the record holds the regular gas left before the charge that failed. The reservoir pays
/// nothing, part of the state charge, all of it, or all of it and part of the history charge.
#[test]
fn test_return_create_records_its_failed_state_and_history_charges() {
    let mut params = GasParams::new_spec(SpecId::AMSTERDAM);
    params.override_gas([(GasId::code_deposit_history_gas(), 1_000)]);
    let hash = params.keccak256_cost(1);
    let state = params.code_deposit_state_gas(1);
    let history = params.code_deposit_history_gas(1);
    let regular = 100;
    assert!(
        regular - hash < (state / 2).min(history / 2),
        "no spill can be paid"
    );

    for reservoir in [0, state / 2, state, state + history / 2] {
        let mut ctx: TestContext = Context::mainnet()
            .with_db(Db::default())
            .modify_cfg_chained(|cfg| {
                cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
                cfg.gas_params
                    .override_gas([(GasId::code_deposit_history_gas(), 1_000)]);
            });
        ctx.journal_mut().load_account(CONTRACT).unwrap();
        let checkpoint = ctx.journal_mut().checkpoint();
        let mut result = InterpreterResult::new(
            InstructionResult::Return,
            Bytes::from_static(&[0]),
            Gas::new_with_regular_gas_and_reservoir(regular, reservoir),
        );

        return_create(&mut ctx, checkpoint, &mut result, CONTRACT);

        assert_eq!(result.result, InstructionResult::OutOfGas, "{reservoir}");
        assert_eq!(result.gas.remaining(), regular - hash, "{reservoir}");
        assert_eq!(
            result.gas.failed_charge_remaining(),
            Some(regular - hash),
            "{reservoir}"
        );
    }
}

/// A cold access the frame cannot pay is refused before its charge: revm skips the load and the
/// frame halts out of gas. The record holds the regular gas left at the refusal, for each
/// instruction that can skip one: an account read (`BALANCE`), `SLOAD`, `SELFDESTRUCT` and a
/// `CALL`'s target. Each child below has 1,000 left at the access, short of its cold cost.
#[test]
fn test_a_skipped_cold_load_records_the_regular_gas_left() {
    let params = GasParams::new_spec(SPEC);
    let left = 1_000;
    assert!(left < params.cold_account_additional_cost());
    assert!(left < params.cold_storage_additional_cost());
    assert!(left < params.selfdestruct_cold_cost());

    let balance = [push(COLD), vec![BALANCE, STOP]].concat();
    let sload = [PUSH0, SLOAD, STOP];
    let selfdestruct = [push(COLD), vec![SELFDESTRUCT]].concat();
    let call_cold = [call(COLD, 0), vec![STOP]].concat();
    for (child, before) in [
        (&balance[..], 3 + static_gas(BALANCE)),
        (&sload[..], 2 + static_gas(SLOAD)),
        (&selfdestruct[..], 3 + static_gas(SELFDESTRUCT)),
        (&call_cold[..], 21 + static_gas(CALL)),
    ] {
        let child_gas = (before + left) as u32;

        assert_called_frame(run(child_gas, child), InstructionResult::OutOfGas, left);
    }
}

/// A `CALL` to an EIP-7702 account, warm itself, loads the account it delegates to as well: a
/// frame that cannot pay that load's warm read is refused before any charge, and one that cannot
/// pay its cold cost skips it. Either way the frame halts out of gas, and the record holds the
/// regular gas left at the refusal.
#[test]
fn test_a_skipped_delegation_load_records_the_regular_gas_left() {
    let params = GasParams::new_spec(SPEC);
    // `BALANCE` warms the delegated account, paying its cold cost; `POP`, then the `CALL`.
    let child = [
        push(DELEGATED),
        vec![BALANCE, POP],
        call(DELEGATED, 0),
        vec![STOP],
    ]
    .concat();
    let before =
        3 + static_gas(BALANCE) + params.cold_account_additional_cost() + 2 + 21 + static_gas(CALL);

    // 50 cannot pay the warm read (100); 1,000 pays it but not the cold cost after it (2,500).
    for left in [50, 1_000] {
        let outcome = Run::on(SPEC)
            .account(DELEGATED, Bytecode::new_eip7702(COLD))
            .call(CHILD, (before + left) as u32, &child);

        assert_called_frame(outcome, InstructionResult::OutOfGas, left);
    }
}

/// An `SSTORE` skips a cold slot it cannot pay for, as the other loads do. With mainnet prices the
/// stipend sentry stops such a frame first; a schedule whose cold storage cost exceeds the
/// stipend reaches the skip, and the record holds the regular gas left at it.
#[test]
fn test_a_skipped_sstore_load_records_the_regular_gas_left() {
    let params = GasParams::new_spec(SPEC);
    let left = 5_000;
    let child_gas = 5 + static_gas(SSTORE) + params.sstore_static_gas() + left;

    let outcome = Run::on(SPEC)
        .price(GasId::cold_storage_cost(), 10_000)
        .call(CHILD, child_gas as u32, &[PUSH1, 1, PUSH0, SSTORE, STOP]);

    assert_called_frame(outcome, InstructionResult::OutOfGas, left);
}

/// A precompile frame whose run needs more gas than it was given halts `PrecompileOOG` with its
/// gas zeroed; the record holds the gas it had, all of it, as nothing ran. One that can pay
/// returns with no record.
#[test]
fn test_a_precompile_out_of_gas_records_its_gas() {
    assert_called_frame(
        Run::on(SPEC).call(ECRECOVER, 100, &[]),
        InstructionResult::PrecompileOOG,
        100,
    );

    let (result, returned) = Run::on(SPEC).call(IDENTITY, 100, &[]);
    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        returned,
        [
            (InstructionResult::Return, None),
            (InstructionResult::Stop, None),
        ]
    );
}

/// The conversion of a precompile's output records the frame's gas limit for an out-of-gas,
/// including a precompile that reports more gas than it was given, and records nothing for a
/// success, a revert or another halt.
#[test]
fn test_a_precompile_output_records_only_an_out_of_gas() {
    let limit = 100;
    let mut revert = PrecompileOutput::new(10, Bytes::new(), 0);
    revert.status = precompile::PrecompileStatus::Revert;
    for (output, expected, record) in [
        (
            PrecompileOutput::halt(PrecompileHalt::OutOfGas, 0),
            InstructionResult::PrecompileOOG,
            Some(limit),
        ),
        (
            PrecompileOutput::new(limit + 1, Bytes::new(), 0),
            InstructionResult::PrecompileOOG,
            Some(limit),
        ),
        (
            PrecompileOutput::halt(PrecompileHalt::Bn254PairLength, 0),
            InstructionResult::PrecompileError,
            None,
        ),
        (
            PrecompileOutput::new(10, Bytes::new(), 0),
            InstructionResult::Return,
            None,
        ),
        (revert, InstructionResult::Revert, None),
    ] {
        let result = precompile_output_to_interpreter_result(output, limit);

        assert_eq!(result.result, expected);
        assert_eq!(result.gas.failed_charge_remaining(), record, "{expected:?}");
    }
}
