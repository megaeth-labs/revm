//! The system call state-gas margin switch ([`Cfg::system_call_state_gas_margin_in_reservoir`]).
//!
//! A system call carries [`SYSTEM_CALL_GAS_LIMIT`]: the base [`SYSTEM_CALL_REGULAR_GAS_LIMIT`]
//! plus a margin sized for [`SYSTEM_MAX_SSTORES_PER_CALL`] fresh storage writes. With the switch
//! off (the default, upstream's behaviour) the whole limit is regular gas. With the switch on
//! and EIP-8037 enabled, the margin is the state-gas reservoir, as EIP-8037 specifies for system
//! calls. Every expected value follows from that split and the flat state gas schedule.
//!
//! [`Cfg::system_call_state_gas_margin_in_reservoir`]: context_interface::Cfg::system_call_state_gas_margin_in_reservoir

use bytecode::opcode::{GAS, INVALID, PUSH0, PUSH1, REVERT, SSTORE, STOP, SUB, SWAP1};
use context::{
    result::{EVMError, ExecutionResult, HaltReason},
    Context, ContextSetters, TxEnv,
};
use context_interface::{Database, Transaction};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{GasTracker, InitialAndFloorGas};
use primitives::{
    address, eip8037, eip8038, hardfork::SpecId, Address, Bytes, StorageKey, TxKind, U256,
};
use revm_handler::{
    ExecuteEvm, Handler, MainBuilder, MainContext, MainnetContext, MainnetEvm, MainnetHandler,
    SystemCallEvm, SystemCallTx, SYSTEM_CALL_GAS_LIMIT, SYSTEM_CALL_REGULAR_GAS_LIMIT,
    SYSTEM_CALL_STATE_GAS_RESERVOIR, SYSTEM_MAX_SSTORES_PER_CALL,
};
use state::{AccountInfo, Bytecode, EvmState};
use std::cell::Cell;

type Db = CacheDB<EmptyDB>;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<<Db as Database>::Error>;

/// The system contract under test.
const SYSTEM_CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");

/// State gas for a 0→x storage write on the flat Amsterdam schedule.
const SSTORE_SET: u64 = eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;
/// The state-gas margin a system call carries above its regular budget.
const MARGIN: u64 = SSTORE_SET * SYSTEM_MAX_SSTORES_PER_CALL;

/// Regular gas of a cold 0→x `SSTORE` on Amsterdam (EIP-8038): the cold slot access and the write.
const SSTORE_SET_REGULAR: u64 = eip8038::COLD_STORAGE_ACCESS + eip8038::STORAGE_WRITE;

/// `SSTORE(0, GAS); STOP`: stores the regular gas left after `GAS` charged its own 2 gas.
const GAS_TO_SLOT: &[u8] = &[GAS, PUSH0, SSTORE, STOP];
/// Regular gas [`GAS_TO_SLOT`] spends: `GAS` and `PUSH0` cost 2 each.
const GAS_TO_SLOT_REGULAR: u64 = 2 + 2 + SSTORE_SET_REGULAR;
/// `SSTORE(0, 1); REVERT(0, 0)`
const SSTORE_AND_REVERT: &[u8] = &[PUSH1, 1, PUSH0, SSTORE, PUSH0, PUSH0, REVERT];
/// Regular gas [`SSTORE_AND_REVERT`] spends: `PUSH1` costs 3, each `PUSH0` 2, `REVERT(0, 0)` 0.
const SSTORE_AND_REVERT_REGULAR: u64 = 3 + 2 + SSTORE_SET_REGULAR + 2 + 2;
/// `SSTORE(0, 1); INVALID`
const SSTORE_AND_HALT: &[u8] = &[PUSH1, 1, PUSH0, SSTORE, INVALID];
/// Regular gas of each write in [`fresh_writes`]: two `PUSH1` at 3 each and the cold 0→x `SSTORE`.
const FRESH_WRITE_REGULAR: u64 = 3 + 3 + SSTORE_SET_REGULAR;
/// Fresh storage writes one past the number the margin is sized for.
const WRITES_PAST_THE_MARGIN: u64 = SYSTEM_MAX_SSTORES_PER_CALL + 1;
/// Follows an earlier `GAS` whose reading is still on the stack: `SSTORE(0, earlier - GAS); STOP`.
/// Stores the regular gas spent between the two readings, this `GAS`'s own 2 gas included.
const SPENT_SINCE_GAS_TO_SLOT: &[u8] = &[GAS, SWAP1, SUB, PUSH0, SSTORE, STOP];

/// `SSTORE(slot, 1)` for each `slot` in `0..writes`.
fn fresh_writes(writes: u64) -> Vec<u8> {
    let mut code = Vec::new();
    for slot in 0..writes {
        let slot = u8::try_from(slot).expect("slot fits in PUSH1");
        code.extend_from_slice(&[PUSH1, 1, PUSH1, slot, SSTORE]);
    }
    code
}

fn db_with(code: &[u8]) -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    db.insert_account_info(
        SYSTEM_CONTRACT,
        AccountInfo::default().with_code(Bytecode::new_raw(Bytes::copy_from_slice(code))),
    );
    db
}

/// A context holding `code` at [`SYSTEM_CONTRACT`] on `spec`, with the switch set to `margin_in_reservoir`.
fn context(code: &[u8], spec: SpecId, margin_in_reservoir: bool) -> TestContext {
    Context::mainnet()
        .with_db(db_with(code))
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(spec);
            cfg.system_call_state_gas_margin_in_reservoir = margin_in_reservoir;
        })
}

/// Runs a system call to [`SYSTEM_CONTRACT`] holding `code` on `spec`.
fn system_call(
    code: &[u8],
    spec: SpecId,
    margin_in_reservoir: bool,
) -> (ExecutionResult, EvmState) {
    let mut evm = context(code, spec, margin_in_reservoir).build_mainnet();
    let output = evm
        .system_call(SYSTEM_CONTRACT, Bytes::new())
        .expect("system call runs");
    (output.result, output.state)
}

/// The value [`GAS_TO_SLOT`] or [`SPENT_SINCE_GAS_TO_SLOT`] stored in slot zero of [`SYSTEM_CONTRACT`].
fn stored_gas(state: &EvmState) -> U256 {
    state[&SYSTEM_CONTRACT]
        .storage
        .get(&StorageKey::ZERO)
        .map(|slot| slot.present_value)
        .expect("slot zero was written")
}

#[test]
fn test_system_call_gas_limit_is_the_regular_budget_plus_the_margin() {
    assert_eq!(SYSTEM_CALL_REGULAR_GAS_LIMIT, 30_000_000);
    assert_eq!(SYSTEM_CALL_STATE_GAS_RESERVOIR, MARGIN);
    assert_eq!(SYSTEM_CALL_GAS_LIMIT, 30_000_000 + MARGIN);
    assert_eq!(SYSTEM_CALL_GAS_LIMIT, 31_566_720);
    assert_eq!(SYSTEM_MAX_SSTORES_PER_CALL, 16);
    assert_eq!(WRITES_PAST_THE_MARGIN, 17);
}

/// Ported from upstream #3892 (`test_system_call_eip8037_state_gas_reservoir`), switch on.
#[test]
fn test_system_call_margin_in_reservoir_leaves_gas_reporting_the_regular_budget() {
    let (result, state) = system_call(GAS_TO_SLOT, SpecId::AMSTERDAM, true);

    assert!(result.is_success(), "{result:?}");
    // `GAS` charges its own 2 gas before pushing the remaining regular gas.
    assert_eq!(
        stored_gas(&state),
        U256::from(SYSTEM_CALL_REGULAR_GAS_LIMIT - 2)
    );
    // The fresh-slot SSTORE is a state-gas charge drawn from the reservoir.
    assert_eq!(result.gas().block_state_gas_used(), SSTORE_SET);
    assert_eq!(result.gas().reservoir_remaining(), MARGIN - SSTORE_SET);
}

/// Switch-off twin: the whole system call gas limit is regular gas.
#[test]
fn test_system_call_margin_stays_regular_gas_by_default() {
    let (result, state) = system_call(GAS_TO_SLOT, SpecId::AMSTERDAM, false);

    assert!(result.is_success(), "{result:?}");
    assert_eq!(stored_gas(&state), U256::from(SYSTEM_CALL_GAS_LIMIT - 2));
    // The state-gas charge spills into regular gas; it is still counted as state gas.
    assert_eq!(result.gas().block_state_gas_used(), SSTORE_SET);
    assert_eq!(result.gas().reservoir_remaining(), 0);
}

#[test]
fn test_system_call_margin_switch_keeps_the_gas_spent_on_success() {
    // The pool the state gas came from does not change what the call spent.
    for margin_in_reservoir in [false, true] {
        let (result, _) = system_call(GAS_TO_SLOT, SpecId::AMSTERDAM, margin_in_reservoir);

        assert!(result.is_success(), "{result:?}");
        assert_eq!(
            result.gas().total_gas_spent(),
            GAS_TO_SLOT_REGULAR + SSTORE_SET
        );
        assert_eq!(result.gas().block_regular_gas_used(), GAS_TO_SLOT_REGULAR);
    }
}

#[test]
fn test_system_call_margin_in_reservoir_takes_back_reverted_state_gas() {
    let (on, _) = system_call(SSTORE_AND_REVERT, SpecId::AMSTERDAM, true);
    let (off, _) = system_call(SSTORE_AND_REVERT, SpecId::AMSTERDAM, false);

    assert!(matches!(on, ExecutionResult::Revert { .. }), "{on:?}");
    assert!(matches!(off, ExecutionResult::Revert { .. }), "{off:?}");
    // The revert returns the slot's state gas to the pool it came from: the reservoir with the
    // switch on, regular gas without it.
    assert_eq!(on.gas().state_gas_spent_final(), 0);
    assert_eq!(on.gas().reservoir_remaining(), MARGIN);
    assert_eq!(off.gas().state_gas_spent_final(), 0);
    assert_eq!(off.gas().reservoir_remaining(), 0);
    // Only the regular gas stays spent.
    assert_eq!(on.gas().total_gas_spent(), SSTORE_AND_REVERT_REGULAR);
    assert_eq!(off.gas().total_gas_spent(), SSTORE_AND_REVERT_REGULAR);
}

#[test]
fn test_system_call_margin_in_reservoir_survives_a_halt() {
    let (result, _) = system_call(SSTORE_AND_HALT, SpecId::AMSTERDAM, true);

    assert!(result.is_halt(), "{result:?}");
    // A halt consumes the regular budget; the reservoir is returned untouched.
    assert_eq!(
        result.gas().total_gas_spent(),
        SYSTEM_CALL_REGULAR_GAS_LIMIT
    );
    assert_eq!(result.gas().reservoir_remaining(), MARGIN);
}

/// Switch-off twin: a halt consumes the whole system call gas limit.
#[test]
fn test_system_call_halt_consumes_the_margin_by_default() {
    let (result, _) = system_call(SSTORE_AND_HALT, SpecId::AMSTERDAM, false);

    assert!(result.is_halt(), "{result:?}");
    assert_eq!(result.gas().total_gas_spent(), SYSTEM_CALL_GAS_LIMIT);
    assert_eq!(result.gas().reservoir_remaining(), 0);
}

// The tests below use 16 fresh storage writes, the number the 1,566,720 margin is sized for
// (16 × 97,920), and 17, one past it. Each write spends 12,106 regular gas.

#[test]
fn test_system_call_margin_in_reservoir_spills_only_the_write_past_the_margin() {
    assert_eq!(FRESH_WRITE_REGULAR, 12_106);

    // Reads `GAS`, runs the writes, then stores the regular gas they spent in slot zero. The first
    // write already changed that slot, so the last `SSTORE` charges no state gas.
    let regular_gas_spent = |writes: u64, margin_in_reservoir: bool| {
        let code = [
            &[GAS],
            fresh_writes(writes).as_slice(),
            SPENT_SINCE_GAS_TO_SLOT,
        ]
        .concat();
        let (result, state) = system_call(&code, SpecId::AMSTERDAM, margin_in_reservoir);

        assert!(result.is_success(), "{result:?}");
        assert_eq!(result.gas().reservoir_remaining(), 0);
        stored_gas(&state)
    };

    // With the switch on, 16 writes use up the reservoir exactly and take no state gas from
    // regular gas: 16 × 12,106 + 2 = 193,698.
    assert_eq!(
        regular_gas_spent(SYSTEM_MAX_SSTORES_PER_CALL, true),
        U256::from(193_698)
    );
    // The 17th write finds the reservoir empty and takes its 97,920 state gas from regular gas:
    // 17 × 12,106 + 97,920 + 2 = 303,724.
    assert_eq!(
        regular_gas_spent(WRITES_PAST_THE_MARGIN, true),
        U256::from(303_724)
    );
    // Switch-off twin: every write takes its state gas from regular gas.
    // 16 × (12,106 + 97,920) + 2 = 1,760,418 and 17 × (12,106 + 97,920) + 2 = 1,870,444.
    assert_eq!(
        regular_gas_spent(SYSTEM_MAX_SSTORES_PER_CALL, false),
        U256::from(1_760_418)
    );
    assert_eq!(
        regular_gas_spent(WRITES_PAST_THE_MARGIN, false),
        U256::from(1_870_444)
    );
}

#[test]
fn test_system_call_margin_switch_keeps_the_gas_spent_past_the_margin() {
    let code = [fresh_writes(WRITES_PAST_THE_MARGIN).as_slice(), &[STOP]].concat();

    // 17 × 12,106 = 205,802 regular gas and 17 × 97,920 = 1,664,640 state gas, 1,870,442 in all.
    // The switch only changes which pool the state gas came from.
    for margin_in_reservoir in [false, true] {
        let (result, _) = system_call(&code, SpecId::AMSTERDAM, margin_in_reservoir);

        assert!(result.is_success(), "{result:?}");
        assert_eq!(result.gas().total_gas_spent(), 1_870_442);
        assert_eq!(result.gas().block_regular_gas_used(), 205_802);
        assert_eq!(result.gas().block_state_gas_used(), 1_664_640);
        assert_eq!(result.gas().reservoir_remaining(), 0);
    }
}

#[test]
fn test_system_call_margin_in_reservoir_takes_back_state_gas_past_the_margin() {
    let code = [
        fresh_writes(WRITES_PAST_THE_MARGIN).as_slice(),
        &[PUSH0, PUSH0, REVERT],
    ]
    .concat();
    let (on, _) = system_call(&code, SpecId::AMSTERDAM, true);
    let (off, _) = system_call(&code, SpecId::AMSTERDAM, false);

    assert!(matches!(on, ExecutionResult::Revert { .. }), "{on:?}");
    assert!(matches!(off, ExecutionResult::Revert { .. }), "{off:?}");
    // With the switch on, the revert gives the 17th write's spilled 97,920 back to regular gas and
    // the other 16 writes' state gas back to the reservoir, which is full again.
    assert_eq!(on.gas().state_gas_spent_final(), 0);
    assert_eq!(on.gas().reservoir_remaining(), 1_566_720);
    assert_eq!(off.gas().state_gas_spent_final(), 0);
    assert_eq!(off.gas().reservoir_remaining(), 0);
    // Only the regular gas stays spent: 17 × 12,106 + 2 + 2 for the `PUSH0`s = 205,806.
    assert_eq!(on.gas().total_gas_spent(), 205_806);
    assert_eq!(off.gas().total_gas_spent(), 205_806);
}

#[test]
fn test_system_call_margin_in_reservoir_survives_a_halt_past_the_margin() {
    let code = [fresh_writes(WRITES_PAST_THE_MARGIN).as_slice(), &[INVALID]].concat();
    let (on, _) = system_call(&code, SpecId::AMSTERDAM, true);
    let (off, _) = system_call(&code, SpecId::AMSTERDAM, false);

    assert!(on.is_halt(), "{on:?}");
    assert!(off.is_halt(), "{off:?}");
    // With the switch on, the halt consumes the 30,000,000 regular budget, the spilled 97,920
    // included, and returns the 1,566,720 reservoir; without it the halt consumes all 31,566,720.
    assert_eq!(on.gas().total_gas_spent(), 30_000_000);
    assert_eq!(on.gas().reservoir_remaining(), 1_566_720);
    assert_eq!(off.gas().total_gas_spent(), 31_566_720);
    assert_eq!(off.gas().reservoir_remaining(), 0);
}

/// Without EIP-8037 there is no reservoir, so the switch has nothing to move.
#[test]
fn test_system_call_margin_switch_needs_eip8037() {
    for margin_in_reservoir in [false, true] {
        let (result, state) = system_call(GAS_TO_SLOT, SpecId::OSAKA, margin_in_reservoir);

        assert!(result.is_success(), "{result:?}");
        assert_eq!(stored_gas(&state), U256::from(SYSTEM_CALL_GAS_LIMIT - 2));
        assert_eq!(result.gas().reservoir_remaining(), 0);
    }
}

/// A transaction does not go through the system call gas, whatever the switch says.
#[test]
fn test_system_call_margin_switch_leaves_transactions_alone() {
    // A regular budget above the system call one, so a leak of the split would show in `GAS`.
    const GAS_LIMIT: u64 = 40_000_000;

    let transact = |margin_in_reservoir: bool| {
        let mut evm = context(GAS_TO_SLOT, SpecId::AMSTERDAM, margin_in_reservoir)
            .modify_cfg_chained(|cfg| cfg.tx_gas_limit_cap = Some(u64::MAX))
            .build_mainnet();
        let tx = TxEnv::builder_for_bench()
            .kind(TxKind::Call(SYSTEM_CONTRACT))
            .gas_price(0)
            .gas_limit(GAS_LIMIT)
            .build_fill();
        evm.transact(tx).expect("transaction runs")
    };
    let on = transact(true);
    let off = transact(false);

    assert!(on.result.is_success(), "{:?}", on.result);
    assert!(stored_gas(&on.state) > U256::from(SYSTEM_CALL_REGULAR_GAS_LIMIT));
    assert_eq!(on.result, off.result);
    assert_eq!(stored_gas(&on.state), stored_gas(&off.state));
}

/// The mainnet handler, counting how often the transaction-level gas is built.
#[derive(Default)]
struct TxGasProbe {
    tx_gas_calls: Cell<u32>,
}

impl Handler for TxGasProbe {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = HaltReason;

    fn tx_gas(&self, evm: &mut TestEvm, init_and_floor_gas: &InitialAndFloorGas) -> GasTracker {
        self.tx_gas_calls.set(self.tx_gas_calls.get() + 1);
        MainnetHandler::<TestEvm, TestError, _>::default().tx_gas(evm, init_and_floor_gas)
    }
}

/// Upstream reverted #3892 because system calls stopped going through `Handler::tx_gas`, which a
/// consumer overrides to reset per-run state. Both settings still build the system call gas there.
#[test]
fn test_system_call_gas_goes_through_tx_gas_with_either_setting() {
    for (margin_in_reservoir, regular_budget) in [
        (false, SYSTEM_CALL_GAS_LIMIT),
        (true, SYSTEM_CALL_REGULAR_GAS_LIMIT),
    ] {
        let mut evm = context(GAS_TO_SLOT, SpecId::AMSTERDAM, margin_in_reservoir).build_mainnet();
        evm.ctx
            .set_tx(TxEnv::new_system_tx(SYSTEM_CONTRACT, Bytes::new()));
        assert_eq!(evm.ctx.tx.gas_limit(), SYSTEM_CALL_GAS_LIMIT);

        let mut probe = TxGasProbe::default();
        let result = probe.run_system_call(&mut evm).expect("system call runs");
        let state = evm.finalize();

        assert_eq!(probe.tx_gas_calls.get(), 1);
        assert!(result.is_success(), "{result:?}");
        assert_eq!(stored_gas(&state), U256::from(regular_budget - 2));
    }
}
