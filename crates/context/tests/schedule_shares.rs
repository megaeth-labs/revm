//! A configuration whose gas schedule lacks the fork's share entries forwards a child frame the
//! whole reservoir and every whole pool, as upstream does: the schedule a consumer builds raw and
//! hands to the configuration, and the one it decodes from a configuration encoded before the
//! entries existed.

use context_interface::{
    cfg::{gas::PoolId, GasParams},
    Cfg,
};
use primitives::hardfork::SpecId;
use revm_context::CfgEnv;
use std::sync::Arc;

/// Upstream's Osaka schedule: the fork's table with nothing above id 49, upstream's last.
fn upstream_table() -> [u64; 256] {
    let mut table = *GasParams::new_spec(SpecId::OSAKA).table();
    table[50..].fill(0);
    table
}

/// Whether `cfg`'s schedule forwards the whole reservoir and every whole pool.
fn forwards_everything(cfg: &CfgEnv) -> bool {
    let params = cfg.gas_params();
    [0, 1, 99, 1_000, u64::MAX].into_iter().all(|amount| {
        params.reservoir_forward_share(amount) == amount
            && PoolId::ALL
                .into_iter()
                .all(|pool| params.pool_forward_share(pool, amount) == amount)
    })
}

/// Each way to give a configuration a schedule other than the spec's default: at construction,
/// with the builder and with the setter.
#[test]
fn test_a_cfg_given_a_raw_schedule_forwards_everything() {
    let raw = GasParams::new(Arc::new(upstream_table()));

    let built = CfgEnv::new_with_spec_and_gas_params(SpecId::OSAKA, raw.clone());
    let chained = CfgEnv::new_with_spec(SpecId::OSAKA).with_gas_params(raw.clone());
    let mut set = CfgEnv::new_with_spec(SpecId::OSAKA);
    set.set_gas_params(raw);

    for (name, cfg) in [("built", built), ("chained", chained), ("set", set)] {
        assert!(forwards_everything(&cfg), "{name}");
    }
}

/// A configuration encoded with upstream's schedule, or with the forward percentage an earlier
/// revision of the shares put at id 254, decodes to one that forwards everything.
#[cfg(feature = "serde")]
#[test]
fn test_a_cfg_encoded_without_the_share_entries_forwards_everything() {
    let mut earlier = upstream_table();
    earlier[254] = 100;
    for (name, table) in [("upstream's", upstream_table()), ("earlier", earlier)] {
        let mut encoded = serde_json::to_value(CfgEnv::new_with_spec(SpecId::OSAKA)).unwrap();
        encoded["gas_params"]["table"] = serde_json::json!(table.to_vec());

        let cfg: CfgEnv = serde_json::from_value(encoded).expect(name);

        assert_eq!(cfg.gas_params().table(), &table, "{name}");
        assert!(forwards_everything(&cfg), "{name}");
    }
}
