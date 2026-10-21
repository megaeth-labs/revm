//! The encoding of a [`GasTracker`].
//!
//! In memory the failed-charge record is a number and a flag, and the flag shares a word with the
//! byte pool's crossing. Encoded, the tracker keeps the fields, names and order the derived
//! encoding had: the failed-charge record is an optional number in its place, and the crossing a
//! `bool` in its own.

use super::{GasTracker, WithheldCrossing};

/// A [`GasTracker`] as it is encoded: its fields in their order, with the defaults an encoding
/// made before a field existed decodes to. Each field of the tracker documents its own.
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
    #[serde(default = "unlimited_bytes")]
    bytes_remaining: u64,
    #[serde(default)]
    bytes_spent: i64,
    #[serde(default)]
    bytes_crossing: bool,
    #[serde(default)]
    history_allowance: u64,
}

/// The pool a tracker starts with, and an encoding that predates the pool decodes to.
const fn unlimited_bytes() -> u64 {
    u64::MAX
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
            bytes_remaining: tracker.bytes_remaining,
            bytes_spent: tracker.bytes_spent,
            bytes_crossing: tracker.bytes_crossing,
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
            bytes_remaining: encoded.bytes_remaining,
            bytes_spent: encoded.bytes_spent,
            bytes_crossing: encoded.bytes_crossing,
            history_allowance: encoded.history_allowance,
        };
        tracker.set_failed_charge_remaining(encoded.failed_charge_remaining);
        tracker
    }
}

#[cfg(test)]
mod tests {
    use super::super::GasTracker;

    /// A tracker with every field away from its default: part of its regular gas withheld, a
    /// crossing and a failed charge recorded, a state charge spilled past the reservoir, a refund,
    /// a pool drawn and a draw refused, and an allowance.
    fn every_field_set() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 400, 300);
        tracker.withhold(100);
        assert!(!tracker.record_regular_cost(350));
        assert!(tracker.record_state_cost(500));
        tracker.set_bytes_remaining(900);
        assert!(tracker.record_bytes(20));
        assert!(!tracker.record_bytes(10_000));
        tracker.set_history_allowance(160);
        tracker.record_refund(-3);
        tracker
    }

    /// The encoding has the fields, the names and the order it had before the failed-charge
    /// record and the crossing shared a word, so a format that encodes by position reads it as
    /// before; the failed-charge record is an optional number, and the crossing a `bool`.
    #[test]
    fn test_the_encoding_keeps_its_fields_and_their_order() {
        assert_eq!(
            serde_json::to_string(&every_field_set()).unwrap(),
            concat!(
                r#"{"gas_limit":1000,"remaining":200,"reservoir":0,"state_gas_spent":500,"#,
                r#""state_gas_spilled":200,"history_gas_spent":0,"refunded":-3,"withheld":0,"#,
                r#""withheld_crossing":{"remaining":400},"failed_charge_remaining":400,"#,
                r#""bytes_remaining":880,"bytes_spent":20,"bytes_crossing":true,"#,
                r#""history_allowance":160}"#,
            )
        );
        assert_eq!(
            serde_json::to_string(&GasTracker::new(1_000, 1_000, 0)).unwrap(),
            concat!(
                r#"{"gas_limit":1000,"remaining":1000,"reservoir":0,"state_gas_spent":0,"#,
                r#""state_gas_spilled":0,"history_gas_spent":0,"refunded":0,"withheld":0,"#,
                r#""withheld_crossing":null,"failed_charge_remaining":null,"#,
                r#""bytes_remaining":18446744073709551615,"bytes_spent":0,"bytes_crossing":false,"#,
                r#""history_allowance":0}"#,
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
