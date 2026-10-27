//! A configuration whose gas schedule lacks the fork's share entry forwards a child frame the
//! whole reservoir, as upstream does: the schedule a consumer builds raw and
//! hands to the configuration, and the one it decodes from a configuration encoded before the
//! entries existed.

use context_interface::{cfg::GasParams, Cfg};
use primitives::hardfork::SpecId;
use revm_context::CfgEnv;
use std::sync::Arc;

/// Upstream's Osaka schedule: the fork's table with nothing above id 49, upstream's last.
fn upstream_table() -> [u64; 256] {
    let mut table = *GasParams::new_spec(SpecId::OSAKA).table();
    table[50..].fill(0);
    table
}

/// Schedules without the share entries, as a configuration could have been encoded: upstream's;
/// with the forward percentage an earlier revision of the fork put at id 254, at its default of
/// 100 and at the 98 a consumer set; and with those a later revision put at ids 254 and 250 to
/// 252, at their default of 100.
#[cfg(feature = "serde")]
fn tables_without_the_share_entries() -> [(&'static str, [u64; 256]); 4] {
    let upstream = upstream_table();
    let mut earlier = upstream;
    earlier[254] = 100;
    let mut earlier_98 = upstream;
    earlier_98[254] = 98;
    let mut later = earlier;
    later[250..=252].fill(100);
    [
        ("upstream's", upstream),
        ("an earlier revision's", earlier),
        ("an earlier revision's, at 98", earlier_98),
        ("a later revision's", later),
    ]
}

/// Whether `cfg`'s schedule forwards the whole reservoir.
fn forwards_everything(cfg: &CfgEnv) -> bool {
    let params = cfg.gas_params();
    [0, 1, 99, 1_000, u64::MAX]
        .into_iter()
        .all(|amount| params.reservoir_forward_share(amount) == amount)
}

/// Each way to give a configuration a schedule other than the spec's default: at construction,
/// with the two builders and with the setter.
#[test]
fn test_a_cfg_given_a_raw_schedule_forwards_everything() {
    let raw = GasParams::new(Arc::new(upstream_table()));

    let built = CfgEnv::new_with_spec_and_gas_params(SpecId::OSAKA, raw.clone());
    let chained = CfgEnv::new_with_spec(SpecId::OSAKA).with_gas_params(raw.clone());
    let respecced =
        CfgEnv::new_with_spec(SpecId::PRAGUE).with_spec_and_gas_params(SpecId::OSAKA, raw.clone());
    let mut set = CfgEnv::new_with_spec(SpecId::OSAKA);
    set.set_gas_params(raw);

    for (name, cfg) in [
        ("built", built),
        ("chained", chained),
        ("respecced", respecced),
        ("set", set),
    ] {
        assert!(forwards_everything(&cfg), "{name}");
    }
}

/// A configuration encoded with a schedule without the share entries decodes to one that forwards
/// everything.
#[cfg(feature = "serde")]
#[test]
fn test_a_cfg_encoded_without_the_share_entries_forwards_everything() {
    for (name, table) in tables_without_the_share_entries() {
        let mut encoded = serde_json::to_value(CfgEnv::new_with_spec(SpecId::OSAKA)).unwrap();
        encoded["gas_params"]["table"] = serde_json::json!(table.to_vec());

        let cfg: CfgEnv = serde_json::from_value(encoded).expect(name);

        assert_eq!(cfg.gas_params().table(), &table, "{name}");
        assert!(forwards_everything(&cfg), "{name}");
    }
}
