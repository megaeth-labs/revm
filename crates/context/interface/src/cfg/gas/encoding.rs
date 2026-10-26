//! The encoding of a [`GasTracker`].
//!
//! In memory the failed-charge record is a number and a flag, and the flag shares a word with the
//! pools' crossings, a bit each. Encoded, the tracker keeps the fields, names and order the derived
//! encoding had: the failed-charge record is an optional number in its place, and the crossings a
//! `bool` per pool in their own.
//!
//! A format that carries field names, such as JSON, decodes an encoding made before a field
//! existed with the field's default. A format that encodes by position, such as bincode, does not:
//! the fields that predate the pools keep their places, but an encoding made before the pools'
//! fields were added lacks them and cannot be read (see [`GasTracker`]).

use super::{GasTracker, PoolId, WithheldCrossing};

/// A [`GasTracker`] as it is encoded: its fields in their order, with the defaults an encoding
/// made before a field existed decodes to in a format that carries field names. Each field of the
/// tracker documents its own.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename = "GasTracker")]
pub(super) struct Encoded {
    gas_limit: u64,
    remaining: u64,
    reservoir: u64,
    state_gas_spent: i64,
    state_gas_spilled: u64,
    #[serde(default)]
    history_gas_spent: i64,
    refunded: i64,
    #[serde(default)]
    withheld: u64,
    #[serde(default)]
    withheld_crossing: Option<WithheldCrossing>,
    #[serde(default)]
    failed_charge_remaining: Option<u64>,
    #[serde(default = "unlimited_pools")]
    pools_remaining: [u64; PoolId::COUNT],
    #[serde(default)]
    pools_spent: [i64; PoolId::COUNT],
    #[serde(default)]
    pool_crossings: [bool; PoolId::COUNT],
    #[serde(default)]
    history_allowance: u64,
}

/// The pools a tracker starts with, and the pools an encoding that predates them decodes to in a
/// format that carries field names.
const fn unlimited_pools() -> [u64; PoolId::COUNT] {
    [u64::MAX; PoolId::COUNT]
}

impl From<GasTracker> for Encoded {
    fn from(tracker: GasTracker) -> Self {
        Self {
            gas_limit: tracker.gas_limit,
            remaining: tracker.remaining,
            reservoir: tracker.reservoir,
            state_gas_spent: tracker.state_gas_spent,
            state_gas_spilled: tracker.state_gas_spilled,
            history_gas_spent: tracker.history_gas_spent,
            refunded: tracker.refunded,
            withheld: tracker.withheld,
            withheld_crossing: tracker.withheld_crossing,
            failed_charge_remaining: tracker.failed_charge_remaining(),
            pools_remaining: tracker.pools_remaining,
            pools_spent: tracker.pools_spent,
            pool_crossings: PoolId::ALL.map(|pool| tracker.pool_crossing(pool)),
            history_allowance: tracker.history_allowance,
        }
    }
}

impl From<Encoded> for GasTracker {
    fn from(encoded: Encoded) -> Self {
        let mut tracker = Self {
            gas_limit: encoded.gas_limit,
            remaining: encoded.remaining,
            reservoir: encoded.reservoir,
            state_gas_spent: encoded.state_gas_spent,
            state_gas_spilled: encoded.state_gas_spilled,
            history_gas_spent: encoded.history_gas_spent,
            refunded: encoded.refunded,
            withheld: encoded.withheld,
            withheld_crossing: encoded.withheld_crossing,
            failed_charge_remaining: 0,
            failed_charge_recorded: false,
            pools_remaining: encoded.pools_remaining,
            pools_spent: encoded.pools_spent,
            pool_crossings: 0,
            history_allowance: encoded.history_allowance,
            start_mark: super::StartMark::NONE,
        };
        tracker.set_failed_charge_remaining(encoded.failed_charge_remaining);
        for (pool, crossing) in PoolId::ALL.into_iter().zip(encoded.pool_crossings) {
            tracker.set_pool_crossing(pool, crossing);
        }
        tracker
    }
}

#[cfg(test)]
mod tests {
    use super::super::{GasTracker, PoolId};

    /// A tracker with every field away from its default: part of its regular gas withheld, a
    /// crossing and a failed charge recorded, a state charge spilled past the reservoir, a refund,
    /// a pool drawn, a draw refused on another, a pool refilled below zero, and an allowance.
    fn every_field_set() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 400, 300);
        tracker.withhold(100);
        assert!(!tracker.record_regular_cost(350));
        assert!(tracker.record_state_cost(500));
        let [first, second, third] = PoolId::ALL;
        tracker.set_pool_remaining(first, 900);
        assert!(tracker.record_pool(first, 20));
        tracker.set_pool_remaining(second, 7);
        assert!(!tracker.record_pool(second, 10_000));
        tracker.set_pool_remaining(third, 0);
        tracker.refill_pool(third, 5);
        tracker.set_history_allowance(160);
        tracker.record_refund(-3);
        tracker
    }

    /// The encoding has the fields, the names and the order it had before the failed-charge
    /// record and the crossings shared a word, so a format that encodes by position finds every
    /// field in the place it had then; the failed-charge record is an optional number, and the
    /// crossings a `bool` per pool. Such a format still cannot read an encoding made before the
    /// pools' fields were added, which lacks them.
    #[test]
    fn test_the_encoding_keeps_its_fields_and_their_order() {
        assert_eq!(
            serde_json::to_string(&every_field_set()).unwrap(),
            concat!(
                r#"{"gas_limit":1000,"remaining":200,"reservoir":0,"state_gas_spent":500,"#,
                r#""state_gas_spilled":200,"history_gas_spent":0,"refunded":-3,"withheld":0,"#,
                r#""withheld_crossing":{"remaining":400},"failed_charge_remaining":400,"#,
                r#""pools_remaining":[880,7,5],"pools_spent":[20,0,-5],"#,
                r#""pool_crossings":[false,true,false],"history_allowance":160}"#,
            )
        );
        assert_eq!(
            serde_json::to_string(&GasTracker::new(1_000, 1_000, 0)).unwrap(),
            concat!(
                r#"{"gas_limit":1000,"remaining":1000,"reservoir":0,"state_gas_spent":0,"#,
                r#""state_gas_spilled":0,"history_gas_spent":0,"refunded":0,"withheld":0,"#,
                r#""withheld_crossing":null,"failed_charge_remaining":null,"#,
                r#""pools_remaining":[18446744073709551615,18446744073709551615,"#,
                r#"18446744073709551615],"pools_spent":[0,0,0],"#,
                r#""pool_crossings":[false,false,false],"history_allowance":0}"#,
            )
        );
    }

    /// Every field round-trips, a failed charge of zero apart from none.
    #[test]
    fn test_the_encoding_round_trips() {
        let mut zero_left = GasTracker::new(1_000, 0, 0);
        assert!(!zero_left.record_regular_cost(1));
        for tracker in [every_field_set(), zero_left, GasTracker::default()] {
            let encoded = serde_json::to_string(&tracker).unwrap();
            assert_eq!(
                serde_json::from_str::<GasTracker>(&encoded).unwrap(),
                tracker,
                "{encoded}"
            );
        }
    }
}
