//! The record a failed regular charge leaves on a [`GasTracker`].
//!
//! A frame whose regular charge fails halts, and an `OutOfGas` halt zeroes the frame's regular
//! gas before its result reaches the handler. The record keeps what the frame had left before the
//! charge, so a consumer can read it from the frame's result.

use super::{GasTracker, WithheldCrossing};
use core::num::NonZeroU64;

impl GasTracker {
    /// Returns the regular gas left, spendable and withheld parts together, before the last
    /// regular charge that failed on this tracker, or `None` if none failed.
    ///
    /// Every failed regular charge records it, through
    /// [`record_regular_cost`](Self::record_regular_cost) or
    /// [`record_cost_unsafe`](Self::record_cost_unsafe): one the withheld part would have paid,
    /// which also leaves a [`WithheldCrossing`] holding the same amount, one beyond the whole
    /// regular gas, and one with no regular gas left, which records `Some(0)`. The failed charge
    /// took nothing, so the record is all the regular gas the frame had; for the unchecked charge
    /// it is read before the spendable part wraps.
    ///
    /// A regular charge that succeeds leaves the record as it is, and so does every other method.
    /// A deduction that is not a regular charge records nothing when it fails: the gas forwarded
    /// to a child, and a state or history charge spilling past the reservoir.
    ///
    /// The record is never cleared. It survives [`spend_all`](Self::spend_all), so it can be read
    /// after the halt the failed charge caused, whatever the halt did to the regular gas, and a
    /// later failed charge replaces it. A child frame starts with none, and a parent does not take
    /// over its child's. A frame whose regular charge fails halts, except before Homestead, where
    /// `return_create` deploys empty code when the frame cannot pay the code deposit: there the
    /// record is left on a creation that returns successfully.
    #[inline]
    pub const fn failed_charge_remaining(&self) -> Option<u64> {
        self.failed_charge_remaining
    }

    /// Records a failed regular charge of `cost` against `spendable`: the regular gas left,
    /// `spendable` plus the withheld part, as
    /// [`failed_charge_remaining`](Self::failed_charge_remaining), and as a [`WithheldCrossing`]
    /// as well when the withheld part would have paid the charge.
    ///
    /// Only called on the failure path of a regular charge, where `cost > spendable`, so the
    /// charge's success path does not pay for it. `spendable` is the spendable part before the
    /// charge: [`record_cost_unsafe`](Self::record_cost_unsafe) has already wrapped it.
    #[cold]
    #[inline(never)]
    pub(super) const fn regular_charge_failed(&mut self, cost: u64, spendable: u64) {
        let remaining = spendable.saturating_add(self.withheld);
        self.failed_charge_remaining = Some(remaining);
        // `cost > spendable`, so a withheld part that covers the difference is never zero, and
        // neither is the regular gas left.
        if cost - spendable <= self.withheld {
            if let Some(remaining) = NonZeroU64::new(remaining) {
                self.withheld_crossing = Some(WithheldCrossing::with_remaining(remaining));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GasTracker, WithheldCrossing};
    use core::num::NonZeroU64;

    /// A tracker with a gas limit and regular gas of 1,000, a reservoir of 300, and 600 of the
    /// regular gas withheld: 400 spendable, 1,000 remaining.
    fn withheld_600() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 1_000, 300);
        tracker.withhold(600);
        tracker
    }

    /// Fails a charge of `cost` on `tracker`, through the checked or the unchecked charge.
    fn fail(tracker: &mut GasTracker, cost: u64, unchecked: bool) {
        if unchecked {
            assert!(tracker.record_cost_unsafe(cost), "the charge fails");
        } else {
            assert!(!tracker.record_regular_cost(cost), "the charge fails");
        }
    }

    fn crossing(tracker: &GasTracker) -> Option<u64> {
        tracker
            .withheld_crossing()
            .map(|crossing| crossing.remaining())
    }

    /// A tracker starts with no record.
    #[test]
    fn test_a_new_tracker_has_no_record() {
        assert_eq!(
            GasTracker::new(1_000, 1_000, 0).failed_charge_remaining(),
            None
        );
        assert_eq!(GasTracker::default().failed_charge_remaining(), None);
    }

    /// A charge that succeeds records nothing, and leaves an earlier record as it is.
    #[test]
    fn test_a_charge_that_succeeds_leaves_the_record_alone() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        assert!(tracker.record_regular_cost(400));
        assert!(!tracker.record_cost_unsafe(100));
        assert_eq!(tracker.failed_charge_remaining(), None);

        fail(&mut tracker, 501, false);
        assert_eq!(tracker.failed_charge_remaining(), Some(500));

        assert!(tracker.record_regular_cost(200));
        assert!(!tracker.record_cost_unsafe(300));
        assert_eq!(tracker.remaining(), 0);
        assert_eq!(tracker.failed_charge_remaining(), Some(500));
    }

    /// A plain out-of-gas, with nothing withheld, records the regular gas left before the charge,
    /// and no crossing. The unchecked charge records it before the spendable part wraps.
    #[test]
    fn test_a_plain_out_of_gas_records_the_regular_gas_left() {
        for unchecked in [false, true] {
            let mut tracker = GasTracker::new(1_000, 400, 0);

            fail(&mut tracker, 401, unchecked);

            assert_eq!(
                tracker.failed_charge_remaining(),
                Some(400),
                "unchecked: {unchecked}"
            );
            assert_eq!(tracker.withheld_crossing(), None, "unchecked: {unchecked}");
        }
    }

    /// A charge that fails with no regular gas left records zero, not nothing.
    #[test]
    fn test_a_failure_with_nothing_left_records_zero() {
        for unchecked in [false, true] {
            let mut tracker = GasTracker::new(1_000, 0, 300);

            fail(&mut tracker, 1, unchecked);

            assert_eq!(
                tracker.failed_charge_remaining(),
                Some(0),
                "unchecked: {unchecked}"
            );
        }
    }

    /// A crossing records the regular gas left, both parts, and the crossing holds the same.
    #[test]
    fn test_a_crossing_records_the_regular_gas_left() {
        for unchecked in [false, true] {
            let mut tracker = withheld_600();

            fail(&mut tracker, 1_000, unchecked);

            assert_eq!(
                tracker.failed_charge_remaining(),
                Some(1_000),
                "unchecked: {unchecked}"
            );
            assert_eq!(crossing(&tracker), Some(1_000), "unchecked: {unchecked}");
        }
    }

    /// A charge beyond the whole regular gas records the regular gas left, both parts, and no
    /// crossing.
    #[test]
    fn test_a_failure_beyond_the_withheld_part_records_the_regular_gas_left() {
        for unchecked in [false, true] {
            let mut tracker = withheld_600();

            fail(&mut tracker, 1_001, unchecked);

            assert_eq!(
                tracker.failed_charge_remaining(),
                Some(1_000),
                "unchecked: {unchecked}"
            );
            assert_eq!(tracker.withheld_crossing(), None, "unchecked: {unchecked}");
        }
    }

    /// The halt after a failed charge zeroes the regular gas and keeps the record.
    #[test]
    fn test_the_record_survives_spend_all() {
        let mut tracker = GasTracker::new(1_000, 400, 0);
        fail(&mut tracker, 401, true);

        tracker.spend_all();

        assert_eq!(tracker.remaining(), 0);
        assert_eq!(tracker.failed_charge_remaining(), Some(400));
    }

    /// A later failure replaces the record, while the crossing record keeps the last crossing.
    #[test]
    fn test_a_later_failure_replaces_the_record() {
        let mut tracker = withheld_600();
        fail(&mut tracker, 500, false);
        assert_eq!(tracker.failed_charge_remaining(), Some(1_000));
        assert_eq!(crossing(&tracker), Some(1_000));

        // A forward draws the withheld part down before a failure beyond the whole regular gas.
        assert!(tracker.record_withheld_first_cost(100));
        fail(&mut tracker, 5_000, false);

        assert_eq!(tracker.failed_charge_remaining(), Some(900));
        assert_eq!(crossing(&tracker), Some(1_000), "the crossing stays");
    }

    /// A deduction that is not a regular charge records nothing when it fails.
    #[test]
    fn test_other_deductions_record_nothing() {
        let mut tracker = GasTracker::new(1_000, 100, 0);

        assert!(!tracker.record_withheld_first_cost(101));
        assert!(!tracker.record_state_cost(101));
        assert!(!tracker.record_history_cost(101));

        assert_eq!(tracker.failed_charge_remaining(), None);
    }

    /// Clearing or setting the crossing record leaves the failed-charge record as it is.
    #[test]
    fn test_the_crossing_methods_leave_the_record_alone() {
        let mut tracker = withheld_600();
        fail(&mut tracker, 500, false);

        tracker.clear_withheld_crossing();
        assert_eq!(tracker.failed_charge_remaining(), Some(1_000));

        let set = WithheldCrossing::with_remaining(NonZeroU64::new(7).unwrap());
        tracker.set_withheld_crossing(Some(set));
        assert_eq!(tracker.failed_charge_remaining(), Some(1_000));
    }

    /// The record round-trips, a zero included, and encodings made before it existed decode with
    /// none.
    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_round_trip_and_older_payloads() {
        for regular in [0, 400] {
            let mut tracker = GasTracker::new(1_000, regular, 0);
            fail(&mut tracker, regular + 1, false);
            let encoded = serde_json::to_value(tracker).unwrap();
            assert_eq!(encoded["failed_charge_remaining"], regular);
            assert_eq!(
                serde_json::from_value::<GasTracker>(encoded.clone()).unwrap(),
                tracker
            );

            let mut older = encoded;
            older
                .as_object_mut()
                .unwrap()
                .remove("failed_charge_remaining");
            let decoded = serde_json::from_value::<GasTracker>(older).unwrap();
            assert_eq!(decoded.failed_charge_remaining(), None);
            assert_eq!(decoded.remaining(), regular);
        }
    }
}
