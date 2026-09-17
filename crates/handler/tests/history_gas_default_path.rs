//! The history gas component changes no number for a consumer that does not use it.
//!
//! Every schedule here prices deposited code's history bytes at zero, and nothing in this
//! workspace books history gas otherwise, so representative transactions report the gas they
//! reported before the component existed. The expected figures were taken from `main` at
//! `f120e3ed`, the commit this component was built on.

use bytecode::opcode::{
    CALL, CODECOPY, CREATE, INVALID, LOG1, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3, PUSH32,
    RETURN, REVERT, SSTORE, STOP,
};
use context::{
    result::{ExecutionResult, Output},
    Context, TxEnv,
};
use context_interface::{
    cfg::{GasId, GasParams},
    result::ResultGas,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use primitives::{hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{ExecuteEvm, MainBuilder, MainContext};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;

const CONTRACT: Address = address(0xc0de);
const OK_CHILD: Address = address(0xc100);
const REVERTING_CHILD: Address = address(0xc1fd);
const HALTING_CHILD: Address = address(0xc1fe);

const GAS_LIMIT: u64 = 3_000_000;
/// Below [`GAS_LIMIT`] on Amsterdam, so the transactions there start with a reservoir.
const AMSTERDAM_CAP: u64 = 2_000_000;

const fn address(tail: u16) -> Address {
    let mut bytes = [0; 20];
    let [hi, lo] = tail.to_be_bytes();
    bytes[18] = hi;
    bytes[19] = lo;
    Address::new(bytes)
}

const fn sstore(slot: u8, value: u8) -> [u8; 5] {
    [PUSH1, value, PUSH1, slot, SSTORE]
}

/// `CALL(100_000, to, 0, 0, 0, 0, 0)`, dropping the success flag.
fn call(to: Address) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20];
    code.extend_from_slice(to.as_slice());
    code.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, CALL, POP]);
    code
}

/// Initcode that writes a slot and deploys 32 `JUMPDEST` bytes.
fn initcode() -> Vec<u8> {
    let mut code = sstore(0, 1).to_vec();
    code.push(PUSH32);
    code.extend_from_slice(&[0x5b; 32]);
    code.extend_from_slice(&[PUSH0, MSTORE, PUSH1, 32, PUSH0, RETURN]);
    code
}

/// Restores a slot, writes another, logs, calls a child that succeeds, one that reverts and one
/// that halts (each after a write), then creates a contract from the initcode it carries.
fn contract() -> Vec<u8> {
    let mut code = Vec::new();
    code.extend_from_slice(&sstore(1, 1));
    code.extend_from_slice(&sstore(1, 0));
    code.extend_from_slice(&sstore(2, 1));
    code.extend_from_slice(&[
        PUSH1, 0x2a, PUSH0, MSTORE, PUSH1, 0x07, PUSH1, 32, PUSH0, LOG1,
    ]);
    for child in [OK_CHILD, REVERTING_CHILD, HALTING_CHILD] {
        code.extend_from_slice(&call(child));
    }
    let initcode = initcode();
    let len = initcode.len() as u8;
    // PUSH1 len, PUSH1 offset, PUSH0, CODECOPY, PUSH1 len, PUSH0, PUSH0, CREATE, POP, STOP
    let offset = (code.len() + 13) as u8;
    code.extend_from_slice(&[
        PUSH1, len, PUSH1, offset, PUSH0, CODECOPY, PUSH1, len, PUSH0, PUSH0, CREATE, POP, STOP,
    ]);
    assert_eq!(code.len(), offset as usize);
    code.extend_from_slice(&initcode);
    code
}

fn db() -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    let mut ok = sstore(0, 1).to_vec();
    ok.push(STOP);
    let mut reverting = sstore(0, 1).to_vec();
    reverting.extend_from_slice(&[PUSH0, PUSH0, REVERT]);
    let mut halting = sstore(0, 1).to_vec();
    halting.push(INVALID);
    for (address, code) in [
        (CONTRACT, contract()),
        (OK_CHILD, ok),
        (REVERTING_CHILD, reverting),
        (HALTING_CHILD, halting),
    ] {
        let code = Bytecode::new_raw(Bytes::from(code));
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    db
}

fn transact(spec: SpecId, kind: TxKind, data: Vec<u8>) -> ExecutionResult {
    let mut evm = Context::mainnet()
        .with_db(db())
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(spec);
            if spec == SpecId::AMSTERDAM {
                cfg.tx_gas_limit_cap = Some(AMSTERDAM_CAP);
            }
        })
        .build_mainnet();
    let tx = TxEnv::builder_for_bench()
        .kind(kind)
        .data(Bytes::from(data))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill();
    evm.transact(tx).expect("transaction runs").result
}

/// `(spec, the call's gas, the create's gas)`, as `main` reported them.
///
/// `ResultGas::new_with_state_gas(total_gas_spent, refunded, floor_gas, state_gas_spent)`.
const EXPECTED: [(SpecId, ResultGas, ResultGas); 3] = [
    (
        SpecId::PRAGUE,
        ResultGas::new_with_state_gas(278_968, 19_900, 21_000, 0),
        ResultGas::new_with_state_gas(82_218, 0, 22_730, 0),
    ),
    (
        SpecId::OSAKA,
        ResultGas::new_with_state_gas(278_968, 19_900, 21_000, 0),
        ResultGas::new_with_state_gas(82_218, 0, 22_730, 0),
    ),
    (
        SpecId::AMSTERDAM,
        ResultGas::new_with_state_gas(724_094, 10_000, 15_000, 526_320),
        ResultGas::new_with_state_gas(367_304, 0, 26_816, 330_480),
    ),
];

#[test]
fn test_no_schedule_prices_deposited_code_as_history() {
    let specs: Vec<SpecId> = (0..=u8::MAX).filter_map(SpecId::try_from_u8).collect();
    assert!(specs.contains(&SpecId::AMSTERDAM));
    for spec in specs {
        let params = GasParams::new_spec(spec);
        assert_eq!(params.get(GasId::code_deposit_history_gas()), 0, "{spec:?}");
        assert_eq!(params.code_deposit_history_gas(0x10000), 0, "{spec:?}");
    }
}

#[test]
fn test_result_gas_is_what_main_reported() {
    for (spec, call_gas, create_gas) in EXPECTED {
        let call = transact(spec, TxKind::Call(CONTRACT), Vec::new());
        let ExecutionResult::Success { gas, logs, .. } = &call else {
            panic!("{spec:?} call succeeds: {call:?}");
        };
        assert_eq!(logs.len(), 1, "{spec:?}");
        assert_eq!(*gas, call_gas, "{spec:?} call");

        let create = transact(spec, TxKind::Create, initcode());
        let ExecutionResult::Success {
            gas,
            output: Output::Create(code, Some(_)),
            ..
        } = &create
        else {
            panic!("{spec:?} create succeeds: {create:?}");
        };
        assert_eq!(code.len(), 32, "{spec:?}");
        assert_eq!(*gas, create_gas, "{spec:?} create");
    }
}
