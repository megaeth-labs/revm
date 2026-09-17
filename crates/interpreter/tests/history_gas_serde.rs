//! Serde encoding of the history gas counter on [`GasTracker`], and so on [`Gas`].
//!
//! New encodings carry the counter. Encodings made before it existed lack it and still decode, to
//! zero, which is what a tracker that never charged history gas carries.
#![cfg(feature = "serde")]

use revm_interpreter::{Gas, GasTracker};
use serde_json::Value;

const FIELD: &str = "history_gas_spent";

/// History gas charged before encoding; larger than the reservoir, so part of it spills.
const CHARGED: u64 = 30_000;

/// A tracker that has charged [`CHARGED`] history gas.
fn charged_tracker() -> GasTracker {
    let mut tracker = GasTracker::new(100_000, 100_000, 20_000);
    assert!(tracker.record_history_cost(CHARGED));
    tracker
}

/// `tracker` with its history counter back at zero and everything else untouched.
const fn without_history(mut tracker: GasTracker) -> GasTracker {
    tracker.set_history_gas_spent(0);
    tracker
}

/// Removes [`FIELD`] from an encoded tracker and returns its value.
fn remove_field(tracker: &mut Value) -> Option<Value> {
    let Value::Object(fields) = tracker else {
        panic!("expected a JSON object, got {tracker}");
    };
    fields.remove(FIELD)
}

#[test]
fn test_gas_tracker_decodes_with_and_without_the_history_counter() {
    let tracker = charged_tracker();
    let mut encoded = serde_json::to_value(tracker).unwrap();
    assert_eq!(
        serde_json::from_value::<GasTracker>(encoded.clone()).unwrap(),
        tracker
    );

    assert_eq!(remove_field(&mut encoded), Some(Value::from(CHARGED)));
    assert_eq!(
        serde_json::from_value::<GasTracker>(encoded).unwrap(),
        without_history(tracker)
    );
}

#[test]
fn test_gas_decodes_with_and_without_the_history_counter() {
    let mut gas = Gas::new_with_regular_gas_and_reservoir(100_000, 20_000);
    assert!(gas.record_history_cost(CHARGED));
    let mut encoded = serde_json::to_value(gas).unwrap();
    assert_eq!(serde_json::from_value::<Gas>(encoded.clone()).unwrap(), gas);

    let Value::Object(fields) = &mut encoded else {
        panic!("expected a JSON object, got {encoded}");
    };
    let tracker = fields.get_mut("tracker").expect("Gas encodes its tracker");
    assert_eq!(remove_field(tracker), Some(Value::from(CHARGED)));

    let mut expected = gas;
    *expected.tracker_mut() = without_history(*gas.tracker());
    assert_eq!(serde_json::from_value::<Gas>(encoded).unwrap(), expected);
}

/// A child frame that refilled history gas its parent charged encodes a negative net.
#[test]
fn test_a_negative_net_round_trips() {
    let mut tracker = GasTracker::new(100_000, 100_000, 0);
    tracker.refill_history(CHARGED);
    assert_eq!(tracker.history_gas_spent(), -(CHARGED as i64));

    let encoded = serde_json::to_value(tracker).unwrap();
    assert_eq!(encoded[FIELD], Value::from(-(CHARGED as i64)));
    assert_eq!(
        serde_json::from_value::<GasTracker>(encoded).unwrap(),
        tracker
    );
}
