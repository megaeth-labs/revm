//! The record a failed charge leaves on a [`GasTracker`].
//!
//! A frame whose charge fails halts out of gas, and an `OutOfGas` halt zeroes the frame's regular
//! gas before its result reaches the handler. The record keeps what the frame had left before the
//! charge, so a consumer can read it from the frame's result.

use super::{GasTracker, WithheldCrossing};
use core::num::NonZeroU64;

impl GasTracker {
    /// Returns the regular gas left, spendable and withheld parts together, before the last
    /// charge that failed on this tracker, or `None` if none failed.
    ///
    /// Every deduction that fails records it:
    ///
    /// - a regular charge, through [`record_regular_cost`](Self::record_regular_cost) or
    ///   [`record_cost_unsafe`](Self::record_cost_unsafe): one the withheld part would have paid,
    ///   which also leaves a [`WithheldCrossing`] holding the same amount, and one beyond the whole
    ///   regular gas. The unchecked charge records the regular gas left before the spendable part
    ///   wraps;
    /// - a state or history charge whose spill past the reservoir regular gas cannot pay, through
    ///   [`record_state_cost`](Self::record_state_cost) or
    ///   [`record_history_cost`](Self::record_history_cost), whether the reservoir paid nothing or
    ///   part of it. The record is regular gas alone; the reservoir is left as it was;
    /// - a forward to a child that regular gas cannot pay, through
    ///   [`record_withheld_first_cost`](Self::record_withheld_first_cost), which only a call
    ///   before EIP-150 can fail.
    ///
    /// The interpreter records it too when it halts a frame out of gas with no record there: a
    /// cold access the instruction refused before its charge, because the frame could not pay it,
    /// an out-of-gas a consumer's instruction returned itself, or one an instruction hook asked
    /// for ([`Host::write_completed`](crate::Host::write_completed),
    /// [`Host::frame_starting`](crate::Host::frame_starting)). The handler records it where a
    /// precompile runs out of gas: the frame's whole gas limit, as nothing ran. A consumer that
    /// builds an out-of-gas result itself can record the same with
    /// [`set_failed_charge_remaining`](Self::set_failed_charge_remaining).
    ///
    /// The failed deduction took nothing, so the record is all the regular gas the frame had; a
    /// failure with nothing left records `Some(0)`. A deduction that succeeds leaves the record as
    /// it is, and so does every credit and setter of the regular gas.
    ///
    /// The tracker never clears the record. It survives [`spend_all`](Self::spend_all), so it can
    /// be read after the halt the failed charge caused, whatever the halt did to the regular gas,
    /// and a later failed charge replaces it. A child frame starts with none, and a parent does not
    /// take over its child's. A frame whose charge fails halts, except before Homestead, where
    /// `return_create` deploys empty code when the frame cannot pay the code deposit: there the
    /// record is left on a creation that returns successfully.
    #[inline]
    pub const fn failed_charge_remaining(&self) -> Option<u64> {
        if self.failed_charge_recorded {
            Some(self.failed_charge_remaining)
        } else {
            None
        }
    }

    /// Sets the record returned by [`failed_charge_remaining`](Self::failed_charge_remaining),
    /// replacing any record already there; `None` clears it. The regular gas is left as it is.
    ///
    /// The tracker records a failed deduction itself. This is for a result produced outside the
    /// tracker that ends a frame out of gas, such as a precompile's or an answer a consumer builds:
    /// `remaining` is the regular gas the frame had before the charge that failed. Like a recorded
    /// failure, the record survives [`spend_all`](Self::spend_all).
    #[inline]
    pub const fn set_failed_charge_remaining(&mut self, remaining: Option<u64>) {
        match remaining {
            Some(remaining) => self.record_failed_charge(remaining),
            None => {
                self.failed_charge_remaining = 0;
                self.failed_charge_recorded = false;
            }
        }
    }

    /// Records `remaining` as the regular gas left before a failed charge.
    #[inline]
    const fn record_failed_charge(&mut self, remaining: u64) {
        self.failed_charge_remaining = remaining;
        self.failed_charge_recorded = true;
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
        self.record_failed_charge(remaining);
        // `cost > spendable`, so a withheld part that covers the difference is never zero, and
        // neither is the regular gas left.
        if cost - spendable <= self.withheld {
            if let Some(remaining) = NonZeroU64::new(remaining) {
                self.withheld_crossing = Some(WithheldCrossing::with_remaining(remaining));
            }
        }
    }

    /// Records a failed deduction that draws the withheld part first: a forward, or the spill of
    /// a state or history charge. The tracker is untouched, so the regular gas left is both parts
    /// as they are. Never a crossing: such a deduction fails only beyond the whole regular gas.
    ///
    /// Only called on the failure path of
    /// [`record_withheld_first_cost`](Self::record_withheld_first_cost), so its success path does
    /// not pay for it.
    #[cold]
    #[inline(never)]
    pub(super) const fn withheld_first_cost_failed(&mut self) {
        self.record_failed_charge(self.remaining.saturating_add(self.withheld));
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

    /// A state or history charge whose spill onto regular gas fails records the regular gas left
    /// before it, both parts, whether the reservoir paid nothing or part of the charge. The failed
    /// charge leaves the reservoir and both parts as they were.
    #[test]
    fn test_a_failed_spill_records_the_regular_gas_left() {
        for history in [false, true] {
            for reservoir in [0, 300] {
                // 300 spendable and 100 withheld; the charge leaves a spill of 401 past the
                // reservoir, which the 400 cannot pay.
                let mut tracker = GasTracker::new(1_000, 400, reservoir);
                tracker.withhold(100);
                let charged = if history {
                    tracker.record_history_cost(reservoir + 401)
                } else {
                    tracker.record_state_cost(reservoir + 401)
                };

                let case = format!("history: {history}, reservoir: {reservoir}");
                assert!(!charged, "{case}");
                assert_eq!(tracker.failed_charge_remaining(), Some(400), "{case}");
                assert_eq!(tracker.reservoir(), reservoir, "{case}");
                assert_eq!(
                    (tracker.spendable(), tracker.withheld()),
                    (300, 100),
                    "{case}"
                );
                assert_eq!(tracker.withheld_crossing(), None, "{case}");
            }
        }
    }

    /// A forward the regular gas cannot pay, both parts together, records the regular gas left.
    #[test]
    fn test_a_failed_forward_records_the_regular_gas_left() {
        let mut tracker = withheld_600();

        assert!(!tracker.record_withheld_first_cost(1_001));

        assert_eq!(tracker.failed_charge_remaining(), Some(1_000));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// State and history charges, paid by the reservoir alone or spilling onto regular gas, and
    /// forwards that succeed leave the record empty.
    #[test]
    fn test_deductions_that_succeed_leave_the_record_alone() {
        let mut tracker = withheld_600();

        assert!(tracker.record_state_cost(100));
        assert!(tracker.record_history_cost(100));
        assert!(tracker.record_state_cost(300));
        assert!(tracker.record_history_cost(300));
        assert!(tracker.record_withheld_first_cost(300));

        assert_eq!(
            tracker.state_gas_spilled(),
            500,
            "the second state and history charges spilled"
        );
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

    /// A record set from outside the tracker leaves the regular gas alone, survives the halt, and
    /// is replaced by a later failure; `None` clears it.
    #[test]
    fn test_set_failed_charge_remaining() {
        let mut tracker = GasTracker::new(1_000, 400, 0);

        tracker.set_failed_charge_remaining(Some(400));
        assert_eq!(tracker.failed_charge_remaining(), Some(400));
        assert_eq!(tracker.remaining(), 400);

        tracker.spend_all();
        assert_eq!(tracker.failed_charge_remaining(), Some(400));

        fail(&mut tracker, 1, false);
        assert_eq!(tracker.failed_charge_remaining(), Some(0));

        tracker.set_failed_charge_remaining(None);
        assert_eq!(tracker.failed_charge_remaining(), None);
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
