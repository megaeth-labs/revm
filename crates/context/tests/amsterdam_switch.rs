//! EIP-7708 transfer-log switch below `SpecId::AMSTERDAM`.
//!
//! Independent of the Amsterdam opcode switch. Default (7708 switch off) keeps the
//! spec-only gate. Switch on at Osaka must emit the same transfer log as Amsterdam.
//! These switch-on cases fail if the cfg flag is not copied into the journal.

use primitives::{
    address,
    eip7708::{ETH_TRANSFER_LOG_ADDRESS, ETH_TRANSFER_LOG_TOPIC},
    hardfork::SpecId,
    Address, Log, B256, U256,
};
use revm_context::{database_interface::EmptyDB, CfgEnv, Context, JournalTr};

const FROM: Address = address!("0x1000000000000000000000000000000000000001");
const TO: Address = address!("0x2000000000000000000000000000000000000002");
const VALUE: U256 = U256::from_limbs([1, 0, 0, 0]);

fn transfer_logs(
    spec: SpecId,
    enable_amsterdam_opcodes: bool,
    enable_amsterdam_eip7708: bool,
) -> Vec<Log> {
    let cfg = CfgEnv::new_with_spec(spec)
        .with_enable_amsterdam_opcodes(enable_amsterdam_opcodes)
        .with_enable_amsterdam_eip7708(enable_amsterdam_eip7708);
    let mut ctx: Context = Context::new(EmptyDB::new(), spec).with_cfg(cfg);

    ctx.journaled_state.load_account(FROM).unwrap();
    ctx.journaled_state.load_account(TO).unwrap();
    ctx.journaled_state
        .state
        .get_mut(&FROM)
        .unwrap()
        .info
        .balance = U256::from(1_000);

    let err = ctx.journaled_state.transfer(FROM, TO, VALUE).unwrap();
    assert_eq!(err, None);

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
    let logs = transfer_logs(SpecId::OSAKA, false, false);
    assert!(logs.is_empty());
}

#[test]
fn test_value_transfer_on_osaka_with_switch_matches_amsterdam() {
    let with_switch = transfer_logs(SpecId::OSAKA, false, true);
    let on_amsterdam = transfer_logs(SpecId::AMSTERDAM, false, false);

    assert_eq!(with_switch, on_amsterdam);
    assert_eq!(with_switch, vec![expected_transfer_log()]);
}

#[test]
fn test_amsterdam_with_switch_off_emits_one_7708_log() {
    let logs = transfer_logs(SpecId::AMSTERDAM, false, false);
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0], expected_transfer_log());
}

#[test]
fn test_value_transfer_emits_no_7708_log_on_osaka_when_only_opcode_switch_on() {
    let logs = transfer_logs(SpecId::OSAKA, true, false);
    assert!(logs.is_empty());
}

#[test]
fn test_value_transfer_emits_7708_log_on_osaka_when_only_eip7708_switch_on() {
    let logs = transfer_logs(SpecId::OSAKA, false, true);
    assert_eq!(logs, vec![expected_transfer_log()]);
}
