//! EIP-7708 transfer-log switch below `SpecId::AMSTERDAM`.
//!
//! Independent of table-level Amsterdam opcode activation. Default (7708 switch
//! off) keeps the spec-only gate. Switch on at Osaka must emit the same transfer
//! log as Amsterdam.

use bytecode::{opcode::*, Bytecode};
use interpreter::{
    gas_table, instruction_table,
    interpreter::{EthInterpreter, ExtBytecode},
    InputsImpl, InstructionResult, Interpreter, SharedMemory,
};
use primitives::{
    address,
    eip7708::{ETH_TRANSFER_LOG_ADDRESS, ETH_TRANSFER_LOG_TOPIC},
    hardfork::SpecId,
    Address, Bytes, Log, B256, U256,
};
use revm_context::{database_interface::EmptyDB, CfgEnv, Context, JournalTr};

const FROM: Address = address!("0x1000000000000000000000000000000000000001");
const TO: Address = address!("0x2000000000000000000000000000000000000002");
const VALUE: U256 = U256::from_limbs([1, 0, 0, 0]);

const DUPN_VECTOR: &[u8] = &[
    PUSH1, 0x01, PUSH1, 0x00, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1,
    DUP1, DUP1, DUP1, DUP1, DUPN, 0x80,
];

fn context_with(spec: SpecId, enable_amsterdam_eip7708: bool) -> Context {
    let cfg = CfgEnv::new_with_spec(spec).with_enable_amsterdam_eip7708(enable_amsterdam_eip7708);
    Context::new(EmptyDB::new(), spec).with_cfg(cfg)
}

fn setup_transfer(ctx: &mut Context) {
    ctx.journaled_state.load_account(FROM).unwrap();
    ctx.journaled_state.load_account(TO).unwrap();
    ctx.journaled_state
        .state
        .get_mut(&FROM)
        .unwrap()
        .info
        .balance = U256::from(1_000);
}

fn transfer(ctx: &mut Context) {
    let err = ctx.journaled_state.transfer(FROM, TO, VALUE).unwrap();
    assert_eq!(err, None);
}

fn transfer_logs(spec: SpecId, enable_amsterdam_eip7708: bool) -> Vec<Log> {
    let mut ctx = context_with(spec, enable_amsterdam_eip7708);
    setup_transfer(&mut ctx);
    transfer(&mut ctx);
    ctx.journaled_state.take_logs()
}

fn expected_transfer_log() -> Log {
    Log {
        address: ETH_TRANSFER_LOG_ADDRESS,
        data: primitives::LogData::new(
            vec![
                ETH_TRANSFER_LOG_TOPIC,
                B256::left_padding_from(FROM.as_slice()),
                B256::left_padding_from(TO.as_slice()),
            ],
            VALUE.to_be_bytes::<32>().into(),
        )
        .expect("3 topics is valid"),
    }
}

#[test]
fn test_value_transfer_emits_no_7708_log_on_osaka_when_switch_off() {
    let logs = transfer_logs(SpecId::OSAKA, false);
    assert!(logs.is_empty());
}

#[test]
fn test_value_transfer_on_osaka_with_switch_matches_amsterdam() {
    let with_switch = transfer_logs(SpecId::OSAKA, true);
    let on_amsterdam = transfer_logs(SpecId::AMSTERDAM, false);

    assert_eq!(with_switch, on_amsterdam);
    assert_eq!(with_switch, vec![expected_transfer_log()]);
}

#[test]
fn test_amsterdam_with_switch_off_emits_one_7708_log() {
    let logs = transfer_logs(SpecId::AMSTERDAM, false);
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0], expected_transfer_log());
}

#[test]
fn test_value_transfer_emits_7708_log_on_osaka_when_only_eip7708_switch_on() {
    let logs = transfer_logs(SpecId::OSAKA, true);
    assert_eq!(logs, vec![expected_transfer_log()]);
}

#[test]
fn test_second_tx_still_emits_7708_log_after_commit_tx() {
    let mut ctx = context_with(SpecId::OSAKA, true);
    setup_transfer(&mut ctx);

    transfer(&mut ctx);
    assert_eq!(
        ctx.journaled_state.take_logs(),
        vec![expected_transfer_log()]
    );

    ctx.journaled_state.commit_tx();
    transfer(&mut ctx);
    assert_eq!(
        ctx.journaled_state.take_logs(),
        vec![expected_transfer_log()]
    );
}

#[test]
fn test_modify_cfg_toggles_7708_switch_off_then_on() {
    let mut ctx = context_with(SpecId::OSAKA, true);
    setup_transfer(&mut ctx);

    transfer(&mut ctx);
    assert_eq!(
        ctx.journaled_state.take_logs(),
        vec![expected_transfer_log()]
    );

    ctx.modify_cfg(|cfg| cfg.enable_amsterdam_eip7708 = false);
    transfer(&mut ctx);
    assert!(ctx.journaled_state.take_logs().is_empty());

    ctx.modify_cfg(|cfg| cfg.enable_amsterdam_eip7708 = true);
    transfer(&mut ctx);
    assert_eq!(
        ctx.journaled_state.take_logs(),
        vec![expected_transfer_log()]
    );
}

#[test]
fn test_dupn_not_activated_on_osaka_when_only_eip7708_switch_on() {
    let mut ctx = context_with(SpecId::OSAKA, true);
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(DUPN_VECTOR))),
        InputsImpl::default(),
        false,
        SpecId::OSAKA,
        10_000,
    );
    let table = instruction_table::<EthInterpreter, Context>();
    let gas = gas_table();
    let action = interpreter.run_plain(&table, &gas, &mut ctx);
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::NotActivated)
    );
}

#[cfg(feature = "serde")]
#[test]
fn test_cfg_env_json_without_eip7708_switch_decodes_to_false() {
    let cfg = CfgEnv::new_with_spec(SpecId::OSAKA);
    let mut value = serde_json::to_value(&cfg).expect("CfgEnv serializes");
    let object = value.as_object_mut().expect("CfgEnv JSON is an object");
    assert_eq!(
        object.remove("enable_amsterdam_eip7708"),
        Some(serde_json::Value::Bool(false))
    );

    let decoded: CfgEnv = serde_json::from_value(value).expect("missing field uses serde default");
    assert!(!decoded.enable_amsterdam_eip7708);
}

#[cfg(feature = "serde")]
#[test]
fn test_cfg_env_eip7708_switch_true_roundtrips() {
    let cfg = CfgEnv::new_with_spec(SpecId::OSAKA).with_enable_amsterdam_eip7708(true);
    let json = serde_json::to_string(&cfg).expect("CfgEnv serializes");
    let decoded: CfgEnv = serde_json::from_str(&json).expect("CfgEnv deserializes");
    assert!(decoded.enable_amsterdam_eip7708);
    assert_eq!(decoded, cfg);
}
