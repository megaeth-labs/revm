//! The system call state-gas margin switch on the inspected system call path
//! ([`InspectorHandler::inspect_run_system_call`]), which builds its gas like
//! [`Handler::run_system_call`].
//!
//! [`Handler::run_system_call`]: handler::Handler::run_system_call

use context::{
    result::{EVMError, HaltReason},
    Context, ContextSetters, TxEnv,
};
use database::{CacheDB, EmptyDB};
use handler::{
    EthFrame, ExecuteEvm, Handler, MainBuilder, MainContext, MainnetContext, MainnetEvm,
    MainnetHandler, SystemCallTx, SYSTEM_CALL_GAS_LIMIT, SYSTEM_CALL_REGULAR_GAS_LIMIT,
    SYSTEM_MAX_SSTORES_PER_CALL,
};
use interpreter::{interpreter::EthInterpreter, GasTracker, InitialAndFloorGas};
use primitives::{address, eip8037, hardfork::SpecId, Address, Bytes, StorageKey, U256};
use revm_inspector::{InspectSystemCallEvm, InspectorHandler, NoOpInspector};
use state::{
    bytecode::opcode::{GAS, PUSH0, SSTORE, STOP},
    AccountInfo, Bytecode, EvmState,
};
use std::cell::Cell;

type Db = CacheDB<EmptyDB>;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext, NoOpInspector>;
type TestError = EVMError<core::convert::Infallible>;

/// State gas for a 0→x storage write on the flat Amsterdam schedule.
const SSTORE_SET: u64 = eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;
/// The state-gas margin a system call carries above its regular budget.
const MARGIN: u64 = SSTORE_SET * SYSTEM_MAX_SSTORES_PER_CALL;

/// The system contract under test.
const SYSTEM_CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");

/// `SSTORE(0, GAS); STOP`: stores the regular gas left after `GAS` charged its own 2 gas.
const GAS_TO_SLOT: &[u8] = &[GAS, PUSH0, SSTORE, STOP];

/// An inspected Amsterdam EVM holding [`GAS_TO_SLOT`] at [`SYSTEM_CONTRACT`].
fn evm(margin_in_reservoir: bool) -> TestEvm {
    let mut db = Db::default();
    db.insert_account_info(
        SYSTEM_CONTRACT,
        AccountInfo::default().with_code(Bytecode::new_raw(Bytes::from_static(GAS_TO_SLOT))),
    );
    Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.system_call_state_gas_margin_in_reservoir = margin_in_reservoir;
        })
        .build_mainnet_with_inspector(NoOpInspector)
}

/// The value [`GAS_TO_SLOT`] stored in slot zero of [`SYSTEM_CONTRACT`].
fn stored_gas(state: &EvmState) -> U256 {
    state[&SYSTEM_CONTRACT]
        .storage
        .get(&StorageKey::ZERO)
        .map(|slot| slot.present_value)
        .expect("slot zero was written")
}

#[test]
fn test_inspected_system_call_follows_the_margin_switch() {
    for (margin_in_reservoir, regular_budget, reservoir_remaining) in [
        (false, SYSTEM_CALL_GAS_LIMIT, 0),
        (true, SYSTEM_CALL_REGULAR_GAS_LIMIT, MARGIN - SSTORE_SET),
    ] {
        let output = evm(margin_in_reservoir)
            .inspect_system_call(SYSTEM_CONTRACT, Bytes::new())
            .expect("system call runs");

        assert!(output.result.is_success(), "{:?}", output.result);
        // `GAS` charges its own 2 gas before pushing the remaining regular gas.
        assert_eq!(stored_gas(&output.state), U256::from(regular_budget - 2));
        assert_eq!(output.result.gas().block_state_gas_used(), SSTORE_SET);
        assert_eq!(
            output.result.gas().reservoir_remaining(),
            reservoir_remaining
        );
    }
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
        MainnetHandler::<TestEvm, TestError, EthFrame>::default().tx_gas(evm, init_and_floor_gas)
    }
}

impl InspectorHandler for TxGasProbe {
    type IT = EthInterpreter;
}

#[test]
fn test_inspected_system_call_gas_goes_through_tx_gas_with_either_setting() {
    for (margin_in_reservoir, regular_budget) in [
        (false, SYSTEM_CALL_GAS_LIMIT),
        (true, SYSTEM_CALL_REGULAR_GAS_LIMIT),
    ] {
        let mut evm = evm(margin_in_reservoir);
        evm.ctx
            .set_tx(TxEnv::new_system_tx(SYSTEM_CONTRACT, Bytes::new()));

        let mut probe = TxGasProbe::default();
        let result = probe
            .inspect_run_system_call(&mut evm)
            .expect("system call runs");
        let state = evm.finalize();

        assert_eq!(probe.tx_gas_calls.get(), 1);
        assert!(result.is_success(), "{result:?}");
        assert_eq!(stored_gas(&state), U256::from(regular_budget - 2));
    }
}
