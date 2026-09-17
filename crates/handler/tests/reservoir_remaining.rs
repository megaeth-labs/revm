//! The unspent EIP-8037 reservoir that the mainnet handler reports on [`ResultGas`].
//!
//! The transactions run under Amsterdam with the transaction gas cap below the gas limit, so
//! the part of the limit above the cap is the state gas reservoir. Every expected value follows
//! from that split and the flat state gas schedule, not from the handler's gas tracker.
//!
//! [`ResultGas`]: context_interface::result::ResultGas

use bytecode::opcode::{PUSH0, PUSH1, REVERT, SSTORE, STOP};
use context::{result::ExecutionResult, Context, TxEnv};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use primitives::{address, eip8037, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{ExecuteEvm, MainBuilder, MainContext};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;

/// Contract the transaction calls.
const CONTRACT: Address = address!("0x00000000000000000000000000000000000c0de0");

const GAS_LIMIT: u64 = 10_000_000;
/// The transaction gas cap, which bounds the regular gas budget.
const CAP: u64 = 1_000_000;
/// What the transaction brings to the state gas pool: its gas limit above the cap.
const RESERVOIR: u64 = GAS_LIMIT - CAP;
/// The flat Amsterdam price of a new storage slot.
const SSTORE_SET: u64 = eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;

/// `SSTORE(1, 1); STOP`
const SSTORE_AND_STOP: &[u8] = &[PUSH1, 1, PUSH1, 1, SSTORE, STOP];
/// `SSTORE(1, 1); REVERT(0, 0)`
const SSTORE_AND_REVERT: &[u8] = &[PUSH1, 1, PUSH1, 1, SSTORE, PUSH0, PUSH0, REVERT];

/// Runs a call to [`CONTRACT`] holding `code`, carrying `data`, under Amsterdam with state gas
/// on or off. With state gas off the cap is lifted, as the gas limit may not exceed it then.
fn run(code: &[u8], data: Bytes, state_gas: bool) -> ExecutionResult {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    let code = Bytecode::new_raw(Bytes::copy_from_slice(code));
    db.insert_account_info(
        CONTRACT,
        AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
    );
    let mut evm = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.enable_amsterdam_eip8037 = state_gas;
            cfg.tx_gas_limit_cap = Some(if state_gas { CAP } else { u64::MAX });
        })
        .build_mainnet();
    let tx = TxEnv::builder_for_bench()
        .kind(TxKind::Call(CONTRACT))
        .data(data)
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill();
    evm.transact(tx).unwrap().result
}

#[test]
fn test_result_gas_reports_the_reservoir_left_after_state_gas() {
    let result = run(SSTORE_AND_STOP, Bytes::new(), true);

    assert!(result.is_success());
    // The new slot is paid out of the pool, which is large enough for it.
    assert_eq!(result.gas().state_gas_spent_final(), SSTORE_SET);
    assert_eq!(result.gas().reservoir_remaining(), RESERVOIR - SSTORE_SET);
}

#[test]
fn test_result_gas_reports_the_reservoir_refilled_by_a_revert() {
    let result = run(SSTORE_AND_REVERT, Bytes::new(), true);

    assert!(matches!(result, ExecutionResult::Revert { .. }));
    // The revert returns the slot's state gas to the pool before the result is built.
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(result.gas().reservoir_remaining(), RESERVOIR);
}

#[test]
fn test_result_gas_reports_the_reservoir_before_the_calldata_floor() {
    // Enough non-zero calldata that the EIP-7623 floor exceeds the gas the call uses.
    let result = run(&[STOP], Bytes::from(vec![1u8; 10_000]), true);

    assert!(result.is_success());
    let gas = result.gas();
    assert!(gas.spent_sub_refunded() < gas.floor_gas());
    assert_eq!(gas.tx_gas_used(), gas.floor_gas());
    // The floor settles the pool into the charge afterwards; the reported figure is the pool
    // the transaction left unspent, consistent with `total_gas_spent`.
    assert_eq!(gas.reservoir_remaining(), RESERVOIR);
}

#[test]
fn test_result_gas_reports_no_reservoir_without_state_gas() {
    let result = run(SSTORE_AND_STOP, Bytes::new(), false);

    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(result.gas().reservoir_remaining(), 0);
}
