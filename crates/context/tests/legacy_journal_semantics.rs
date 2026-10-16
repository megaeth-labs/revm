//! The legacy journal semantics switch ([`CfgEnv::legacy_journal_semantics`]).
//!
//! Off (the default) the journal keeps the fork's behaviour: a storage slot warmed while flagged
//! cold inside the current transaction keeps its EIP-2200 `original_value`, and a reverted
//! `CodeChange` restores the code the account had. On, the journal behaves as revm 40.0.3 did:
//! the slot is re-baselined to its `present_value`, and the revert leaves the account with
//! `KECCAK_EMPTY` and no code. The journal learns the switch through the context's cfg sync.

use primitives::{address, hardfork::SpecId, Address, Bytes, KECCAK_EMPTY, U256};
use revm_context::{
    database_interface::EmptyDB, journaled_state::entry::JournalEntry, CfgEnv, Context,
    JournalInner, JournalTr,
};
use state::{Account, AccountInfo, Bytecode, EvmStorageSlot};

const ADDRESS: Address = address!("0x1000000000000000000000000000000000000001");
const KEY: U256 = U256::from_limbs([1, 0, 0, 0]);
const WRITTEN: U256 = U256::from_limbs([42, 0, 0, 0]);

/// A journal holding `ADDRESS` with `KEY` flagged cold inside the current transaction and
/// `present_value != original_value`, the shape a `StorageWarmed` revert or an explicit
/// `mark_cold` leaves behind.
fn journal_with_cold_flagged_slot(legacy: bool) -> JournalInner<JournalEntry> {
    let mut journal = JournalInner::<JournalEntry>::new();
    journal.set_legacy_journal_semantics(legacy);
    let mut account = Account::from(AccountInfo {
        balance: U256::from(1_000),
        nonce: 1,
        code_hash: KECCAK_EMPTY,
        code: Some(Bytecode::default()),
        account_id: None,
    });
    let mut slot = EvmStorageSlot::new_changed(U256::ZERO, WRITTEN, journal.transaction_id);
    slot.mark_cold();
    account.storage.insert(KEY, slot);
    journal.state.insert(ADDRESS, account);
    journal
}

fn original_after_warm(legacy: bool) -> U256 {
    let mut journal = journal_with_cold_flagged_slot(legacy);
    let mut db = EmptyDB::default();
    let load = journal.sload(&mut db, ADDRESS, KEY, false).unwrap();
    assert!(load.is_cold);
    assert_eq!(load.data, WRITTEN);
    let slot = &journal.state[&ADDRESS].storage[&KEY];
    assert!(!slot.is_cold);
    slot.original_value
}

#[test]
fn test_cold_flagged_slot_keeps_its_baseline_when_switch_off() {
    assert_eq!(original_after_warm(false), U256::ZERO);
}

#[test]
fn test_cold_flagged_slot_is_rebaselined_when_switch_on() {
    assert_eq!(original_after_warm(true), WRITTEN);
}

fn code_after_reverted_code_change(legacy: bool) -> Option<Bytecode> {
    let mut journal = JournalInner::<JournalEntry>::new();
    journal.set_legacy_journal_semantics(legacy);
    journal
        .state
        .insert(ADDRESS, Account::new_not_existing(journal.transaction_id));
    let checkpoint = journal.checkpoint();
    let code = Bytecode::new_raw(Bytes::from_static(&[0x60, 0x00]));
    let hash = code.hash_slow();
    journal.set_code_with_hash(ADDRESS, code, hash);
    journal.checkpoint_revert(checkpoint);
    let info = &journal.state[&ADDRESS].info;
    assert_eq!(info.code_hash, KECCAK_EMPTY);
    info.code.clone()
}

#[test]
fn test_reverted_code_change_restores_previous_code_when_switch_off() {
    assert_eq!(
        code_after_reverted_code_change(false),
        Some(Bytecode::default())
    );
}

#[test]
fn test_reverted_code_change_leaves_no_code_when_switch_on() {
    assert_eq!(code_after_reverted_code_change(true), None);
}

/// The same code change through `JournaledAccountTr::set_code`, which records the previous code
/// it replaced.
fn code_after_reverted_account_set_code(legacy: bool) -> Option<Bytecode> {
    use context_interface::journaled_state::account::JournaledAccountTr;

    let mut journal = JournalInner::<JournalEntry>::new();
    journal.set_legacy_journal_semantics(legacy);
    journal
        .state
        .insert(ADDRESS, Account::new_not_existing(journal.transaction_id));
    let checkpoint = journal.checkpoint();
    let code = Bytecode::new_raw(Bytes::from_static(&[0x60, 0x00]));
    let hash = code.hash_slow();
    let mut db = EmptyDB::default();
    journal
        .load_account_mut(&mut db, ADDRESS)
        .unwrap()
        .data
        .set_code(hash, code);
    journal.checkpoint_revert(checkpoint);
    let info = &journal.state[&ADDRESS].info;
    assert_eq!(info.code_hash, KECCAK_EMPTY);
    info.code.clone()
}

#[test]
fn test_reverted_account_set_code_restores_previous_code_when_switch_off() {
    assert_eq!(
        code_after_reverted_account_set_code(false),
        Some(Bytecode::default())
    );
}

#[test]
fn test_reverted_account_set_code_leaves_no_code_when_switch_on() {
    assert_eq!(code_after_reverted_account_set_code(true), None);
}

#[test]
fn test_switch_is_off_by_default() {
    let ctx: Context = Context::new(EmptyDB::new(), SpecId::OSAKA);
    assert!(!ctx.cfg.legacy_journal_semantics);
    assert!(!ctx.journaled_state.inner.legacy_journal_semantics());
}

#[test]
fn test_with_cfg_syncs_the_switch_into_the_journal() {
    let cfg = CfgEnv::new_with_spec(SpecId::OSAKA).with_legacy_journal_semantics(true);
    let ctx: Context = Context::new(EmptyDB::new(), SpecId::OSAKA).with_cfg(cfg);
    assert!(ctx.journaled_state.inner.legacy_journal_semantics());

    let ctx = ctx.with_cfg(CfgEnv::new_with_spec(SpecId::OSAKA));
    assert!(!ctx.journaled_state.inner.legacy_journal_semantics());
}

#[test]
fn test_modify_cfg_syncs_the_switch_into_the_journal() {
    let mut ctx: Context = Context::new(EmptyDB::new(), SpecId::OSAKA);
    ctx.modify_cfg(|cfg| cfg.legacy_journal_semantics = true);
    assert!(ctx.journaled_state.inner.legacy_journal_semantics());
    ctx.modify_cfg(|cfg| cfg.legacy_journal_semantics = false);
    assert!(!ctx.journaled_state.inner.legacy_journal_semantics());
}

#[test]
fn test_assigning_cfg_directly_does_not_reach_the_journal() {
    let mut ctx: Context = Context::new(EmptyDB::new(), SpecId::OSAKA);
    ctx.cfg.legacy_journal_semantics = true;
    assert!(!ctx.journaled_state.inner.legacy_journal_semantics());
    ctx.journaled_state.set_legacy_journal_semantics(true);
    assert!(ctx.journaled_state.inner.legacy_journal_semantics());
}

#[cfg(feature = "serde")]
mod serde_shape {
    use super::*;

    /// A payload written before the switch existed has no key for it and decodes with the
    /// switch off.
    #[test]
    fn test_cfg_env_json_without_legacy_journal_switch_decodes_to_false() {
        let cfg = CfgEnv::new_with_spec(SpecId::OSAKA).with_legacy_journal_semantics(true);
        let mut value = serde_json::to_value(&cfg).expect("CfgEnv serializes");
        let object = value.as_object_mut().expect("CfgEnv JSON is an object");
        assert_eq!(
            object.remove("legacy_journal_semantics"),
            Some(serde_json::Value::Bool(true))
        );

        let decoded: CfgEnv =
            serde_json::from_value(value).expect("missing field uses serde default");
        assert!(!decoded.legacy_journal_semantics);
        assert_eq!(decoded, cfg.with_legacy_journal_semantics(false));
    }

    #[test]
    fn test_journal_json_without_legacy_journal_switch_decodes_to_false() {
        let mut journal = JournalInner::<JournalEntry>::new();
        journal.set_legacy_journal_semantics(true);
        let mut value = serde_json::to_value(&journal).expect("JournalInner serializes");
        let warm = value["warm_addresses"]
            .as_object_mut()
            .expect("WarmAddresses JSON is an object");
        assert_eq!(
            warm.remove("legacy_journal_semantics"),
            Some(serde_json::Value::Bool(true))
        );

        let decoded: JournalInner<JournalEntry> =
            serde_json::from_value(value).expect("missing field uses serde default");
        assert!(!decoded.legacy_journal_semantics());
    }
}
