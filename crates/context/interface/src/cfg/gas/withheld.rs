//! Regular gas withheld from regular charges.
//!
//! The consumer API of the withheld part of a [`GasTracker`]'s regular gas, and the record a
//! regular charge leaves when the withheld part would have paid it. The tracker's own
//! documentation describes how the two parts are read, charged and credited.

use super::GasTracker;
use core::num::NonZeroU64;

/// A failed regular charge that the withheld part of the regular gas would have paid.
///
/// Recorded by [`GasTracker::record_regular_cost`] and [`GasTracker::record_cost_unsafe`] when
/// `spendable < cost <= spendable + withheld`. A charge larger than the whole regular gas records
/// nothing: it would have failed with nothing withheld too.
///
/// The record holds the withheld part at the failed charge ([`withheld`](Self::withheld)). An
/// `OutOfGas` halt zeroes the frame's regular gas, withheld part included, before its result
/// reaches the handler; the record keeps the amount, so a crossing can be settled from the record
/// alone whatever the halt did to the frame's gas.
///
/// A crossing implies `1 <= cost - spendable <= withheld`, so the withheld part is never zero and
/// `Option<WithheldCrossing>` takes no more room than a `u64`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WithheldCrossing {
    /// The withheld part when the charge failed.
    withheld: NonZeroU64,
}

impl WithheldCrossing {
    /// Creates a record of a failed charge that `withheld`, the withheld part at the time, would
    /// have paid.
    ///
    /// The tracker makes its own records. This is for a consumer that marks a result it produced
    /// outside the tracker, such as a precompile's, as a charge that needed withheld gas.
    #[inline]
    pub const fn new(withheld: NonZeroU64) -> Self {
        Self { withheld }
    }

    /// Returns the withheld part when the charge failed. Never zero.
    #[inline]
    pub const fn withheld(&self) -> u64 {
        self.withheld.get()
    }
}

impl GasTracker {
    /// Returns the spendable part of the regular gas: what a regular charge may draw.
    ///
    /// [`remaining`](Self::remaining) is this plus [`withheld`](Self::withheld). With nothing
    /// withheld the two are equal.
    #[inline]
    pub const fn spendable(&self) -> u64 {
        self.remaining
    }

    /// Returns the withheld part of the regular gas: gas held back from regular charges but
    /// still counted by [`remaining`](Self::remaining).
    #[inline]
    pub const fn withheld(&self) -> u64 {
        self.withheld
    }

    /// Moves `min(amount, spendable)` from the spendable part to the withheld part, adding it to
    /// what is already withheld.
    ///
    /// [`remaining`](Self::remaining) is unchanged: only regular charges see the difference.
    /// `amount` is a change, not a target: once gas is withheld, `withhold(remaining() - allowance)`
    /// leaves less than `allowance` spendable. A consumer aiming at an allowance calls
    /// [`limit_spendable`](Self::limit_spendable) instead.
    #[inline]
    pub const fn withhold(&mut self, amount: u64) {
        let amount = if amount < self.remaining {
            amount
        } else {
            self.remaining
        };
        self.remaining -= amount;
        self.withheld += amount;
    }

    /// Makes the spendable part `min(allowance, remaining())` and withholds the rest.
    ///
    /// This is what a consumer aiming at an allowance calls. The outcome depends only on the total
    /// and `allowance`, not on what was withheld before: gas withheld below the allowance is
    /// released and gas spendable above it is withheld, so it can be called again after any
    /// deduction or credit, and a second call with the same allowance changes nothing.
    /// [`remaining`](Self::remaining) is unchanged.
    #[inline]
    pub const fn limit_spendable(&mut self, allowance: u64) {
        let total = self.remaining();
        let spendable = if allowance < total { allowance } else { total };
        self.remaining = spendable;
        self.withheld = total - spendable;
    }

    /// Moves the whole withheld part back to the spendable part.
    ///
    /// [`remaining`](Self::remaining) is unchanged, and [`spendable`](Self::spendable) then
    /// equals it. Releasing is not needed to conserve gas, since a child's withheld part returns
    /// to its parent with the rest of its gas. A consumer that caps a creating frame releases only
    /// after `return_create`, whose code-deposit and code-hash charges draw the spendable part.
    #[inline]
    pub const fn release_withheld(&mut self) {
        self.remaining = self.remaining.wrapping_add(self.withheld);
        self.withheld = 0;
    }

    /// Returns the record of the last failed regular charge the withheld part would have paid,
    /// since the record was last cleared: the withheld part at that charge.
    ///
    /// A later plain out-of-gas leaves the record as it is, and so does
    /// [`spend_all`](Self::spend_all), so it can be read after the halt the failed charge caused,
    /// although the halt zeroes the withheld part itself. A child frame starts with no record,
    /// and a parent does not take over its child's.
    #[inline]
    pub const fn withheld_crossing(&self) -> Option<WithheldCrossing> {
        self.withheld_crossing
    }

    /// Clears the record returned by [`withheld_crossing`](Self::withheld_crossing).
    #[inline]
    pub const fn clear_withheld_crossing(&mut self) {
        self.withheld_crossing = None;
    }

    /// Records a deduction that is not the frame's own regular work, such as the gas forwarded to
    /// a child frame, or the part of a state- or history-gas charge that spills past the
    /// reservoir.
    ///
    /// Draws the withheld part first and the spendable part after it, so it fails only when
    /// [`remaining`](Self::remaining) cannot pay. On failure the tracker is left untouched and no
    /// crossing is recorded. With nothing withheld it deducts exactly what
    /// [`record_regular_cost`](Self::record_regular_cost) deducts.
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_withheld_first_cost(&mut self, cost: u64) -> bool {
        let from_withheld = if cost < self.withheld {
            cost
        } else {
            self.withheld
        };
        if let Some(new_remaining) = self.remaining.checked_sub(cost - from_withheld) {
            self.remaining = new_remaining;
            self.withheld -= from_withheld;
            return true;
        }
        false
    }

    /// Records a failed regular charge of `cost` against `spendable` as a [`WithheldCrossing`]
    /// holding the withheld part, when the withheld part would have paid it.
    ///
    /// Only called on the failure path of a regular charge, where `cost > spendable`, so the
    /// charge's success path does not pay for the check.
    #[cold]
    #[inline(never)]
    pub(super) const fn regular_charge_failed(&mut self, cost: u64, spendable: u64) {
        // `cost > spendable`, so a withheld part that covers the difference is never zero.
        if cost - spendable <= self.withheld {
            if let Some(withheld) = NonZeroU64::new(self.withheld) {
                self.withheld_crossing = Some(WithheldCrossing::new(withheld));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GasTracker, WithheldCrossing};
    use core::{mem::size_of, num::NonZeroU64};

    /// A tracker with a gas limit and regular gas of 1,000, a reservoir of 300, and 600 of the
    /// regular gas withheld: 400 spendable, 1,000 remaining.
    fn withheld_600() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 1_000, 300);
        tracker.withhold(600);
        tracker
    }

    fn parts(tracker: &GasTracker) -> (u64, u64, u64) {
        (tracker.spendable(), tracker.withheld(), tracker.remaining())
    }

    /// The withheld part the tracker's crossing record holds, if any.
    fn recorded(tracker: &GasTracker) -> Option<u64> {
        tracker
            .withheld_crossing()
            .map(|crossing| crossing.withheld())
    }

    /// The record is one word, and an absent record takes no more.
    #[test]
    fn test_an_absent_crossing_takes_no_extra_room() {
        assert_eq!(size_of::<Option<WithheldCrossing>>(), size_of::<u64>());
    }

    /// The tracker is 72 bytes. Above that, copies of the tracker, and of every `Gas` and frame
    /// result that holds one, get markedly more expensive, so a new field is measured before this
    /// changes.
    #[test]
    fn test_the_tracker_is_72_bytes() {
        assert_eq!(size_of::<GasTracker>(), 72);
    }

    /// Withholding moves gas between the parts and leaves the total alone.
    #[test]
    fn test_withhold_moves_spendable_gas_and_keeps_the_total() {
        let tracker = withheld_600();

        assert_eq!(parts(&tracker), (400, 600, 1_000));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// Asking for more than is spendable withholds all of it and no more.
    #[test]
    fn test_withhold_saturates_at_the_spendable_part() {
        let mut tracker = withheld_600();

        tracker.withhold(401);

        assert_eq!(parts(&tracker), (0, 1_000, 1_000));

        tracker.withhold(1);
        assert_eq!(parts(&tracker), (0, 1_000, 1_000));
    }

    /// Limiting the spendable part after a forward drew part of the withheld part restores the
    /// allowance, where withholding the difference from the total would not.
    #[test]
    fn test_limit_spendable_after_a_partial_forward() {
        let mut tracker = withheld_600();
        assert!(tracker.record_withheld_first_cost(500));
        assert_eq!(parts(&tracker), (400, 100, 500));

        let mut delta = tracker;
        delta.withhold(delta.remaining() - 200);
        assert_eq!(
            parts(&delta),
            (100, 400, 500),
            "withhold adds to what is withheld"
        );

        tracker.limit_spendable(200);
        assert_eq!(parts(&tracker), (200, 300, 500));
    }

    /// Limiting the spendable part after a credit withholds what the credit made spendable.
    #[test]
    fn test_limit_spendable_after_a_credit() {
        let mut tracker = withheld_600();
        assert!(tracker.record_withheld_first_cost(700));
        tracker.erase_cost(700);
        assert_eq!(parts(&tracker), (1_000, 0, 1_000));

        tracker.limit_spendable(400);

        assert_eq!(parts(&tracker), (400, 600, 1_000));
    }

    /// An allowance at or above the total releases everything withheld.
    #[test]
    fn test_limit_spendable_above_the_total_releases_it_all() {
        let mut tracker = withheld_600();

        tracker.limit_spendable(5_000);
        assert_eq!(parts(&tracker), (1_000, 0, 1_000));

        let mut tracker = withheld_600();
        tracker.limit_spendable(1_000);
        assert_eq!(parts(&tracker), (1_000, 0, 1_000));
    }

    /// Limiting twice to the same allowance is limiting once, whether the first call withheld
    /// more or released some.
    #[test]
    fn test_limit_spendable_is_idempotent() {
        for allowance in [0, 250, 400, 700, 1_000] {
            let mut tracker = withheld_600();

            tracker.limit_spendable(allowance);
            let once = parts(&tracker);
            tracker.limit_spendable(allowance);

            assert_eq!(parts(&tracker), once, "allowance {allowance}");
            assert_eq!(once, (allowance, 1_000 - allowance, 1_000));
        }
    }

    /// Releasing moves everything back, and releasing again changes nothing.
    #[test]
    fn test_release_withheld_moves_it_all_back() {
        let mut tracker = withheld_600();

        tracker.release_withheld();
        assert_eq!(parts(&tracker), (1_000, 0, 1_000));

        tracker.release_withheld();
        assert_eq!(parts(&tracker), (1_000, 0, 1_000));
    }

    /// A regular charge the spendable part can pay leaves the withheld part alone.
    #[test]
    fn test_record_regular_cost_draws_only_the_spendable_part() {
        let mut tracker = withheld_600();

        assert!(tracker.record_regular_cost(400));

        assert_eq!(parts(&tracker), (0, 600, 600));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// A regular charge above the spendable part fails even though the total could pay it, and
    /// says so: the withheld part it could not draw is recorded.
    #[test]
    fn test_record_regular_cost_records_a_failure_within_the_withheld_part() {
        let mut tracker = withheld_600();

        assert!(!tracker.record_regular_cost(1_000));

        assert_eq!(parts(&tracker), (400, 600, 1_000), "nothing is spent");
        assert_eq!(recorded(&tracker), Some(600));
    }

    /// A record a consumer makes for a withheld part is the one the tracker makes for it.
    #[test]
    fn test_a_constructed_crossing_is_a_recorded_one() {
        let mut tracker = withheld_600();
        assert!(!tracker.record_regular_cost(401));

        let constructed = WithheldCrossing::new(NonZeroU64::new(600).unwrap());

        assert_eq!(constructed.withheld(), 600);
        assert_eq!(tracker.withheld_crossing(), Some(constructed));
    }

    /// A regular charge the whole regular gas cannot pay is a plain out-of-gas.
    #[test]
    fn test_record_regular_cost_records_nothing_beyond_the_withheld_part() {
        let mut tracker = withheld_600();

        assert!(!tracker.record_regular_cost(1_001));

        assert_eq!(parts(&tracker), (400, 600, 1_000));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// A plain out-of-gas after a crossing leaves the crossing in place, and clearing removes it.
    #[test]
    fn test_a_crossing_stays_until_cleared() {
        let mut tracker = withheld_600();
        assert!(!tracker.record_regular_cost(500));

        assert!(!tracker.record_regular_cost(5_000));
        assert_eq!(recorded(&tracker), Some(600), "the crossing stays");

        tracker.clear_withheld_crossing();
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// Of two failures within the withheld part, the record holds the later one.
    #[test]
    fn test_a_second_crossing_replaces_the_first() {
        let mut tracker = withheld_600();
        assert!(!tracker.record_regular_cost(500));
        assert_eq!(recorded(&tracker), Some(600));

        // A forward draws the withheld part down before the second failure.
        assert!(tracker.record_withheld_first_cost(100));
        assert!(!tracker.record_regular_cost(500));

        assert_eq!(recorded(&tracker), Some(500));
    }

    /// With nothing withheld no failure is ever a crossing.
    #[test]
    fn test_nothing_withheld_records_nothing() {
        let mut tracker = GasTracker::new(1_000, 400, 0);

        assert!(!tracker.record_regular_cost(401));
        assert!(tracker.record_cost_unsafe(401));

        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// The step loop's unchecked charge draws the spendable part alone, as the checked one does.
    #[test]
    fn test_record_cost_unsafe_draws_only_the_spendable_part() {
        let mut tracker = withheld_600();

        assert!(!tracker.record_cost_unsafe(400));

        assert_eq!(parts(&tracker), (0, 600, 600));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// The unchecked charge records a crossing when it fails within the withheld part. The halt
    /// that follows zeroes both parts and keeps the record.
    #[test]
    fn test_record_cost_unsafe_records_a_failure_within_the_withheld_part() {
        let mut tracker = withheld_600();

        assert!(tracker.record_cost_unsafe(401));

        assert_eq!(recorded(&tracker), Some(600));
        // The spendable part wraps, as it always did; the total reads the charge taken from it.
        assert_eq!(tracker.remaining(), 599);

        tracker.spend_all();
        assert_eq!(parts(&tracker), (0, 0, 0));
        assert_eq!(
            recorded(&tracker),
            Some(600),
            "the record keeps what the halt zeroed"
        );
    }

    /// The unchecked charge records nothing when the whole regular gas cannot pay it.
    #[test]
    fn test_record_cost_unsafe_records_nothing_beyond_the_withheld_part() {
        let mut tracker = withheld_600();

        assert!(tracker.record_cost_unsafe(1_001));

        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// A forward draws the withheld part first, then the spendable part, and never records.
    #[test]
    fn test_record_withheld_first_cost_draws_the_withheld_part_first() {
        let mut tracker = withheld_600();

        assert!(tracker.record_withheld_first_cost(500));
        assert_eq!(parts(&tracker), (400, 100, 500));

        assert!(tracker.record_withheld_first_cost(300));
        assert_eq!(parts(&tracker), (200, 0, 200));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// A forward the total cannot pay fails and leaves both parts alone.
    #[test]
    fn test_record_withheld_first_cost_fails_only_beyond_the_total() {
        let mut tracker = withheld_600();

        assert!(tracker.record_withheld_first_cost(1_000));
        assert_eq!(parts(&tracker), (0, 0, 0));

        let mut tracker = withheld_600();
        assert!(!tracker.record_withheld_first_cost(1_001));
        assert_eq!(parts(&tracker), (400, 600, 1_000));
        assert_eq!(tracker.withheld_crossing(), None);
    }

    /// A state charge past the reservoir spills onto the withheld part first.
    #[test]
    fn test_a_state_spill_draws_the_withheld_part_first() {
        let mut tracker = withheld_600();

        assert!(tracker.record_state_cost(1_000));

        assert_eq!(tracker.reservoir(), 0);
        assert_eq!(parts(&tracker), (300, 0, 300));
        assert_eq!(tracker.state_gas_spilled(), 700);

        // The total pays what the spendable part alone could not.
        let mut tracker = withheld_600();
        assert!(tracker.record_state_cost(1_300));
        assert_eq!(parts(&tracker), (0, 0, 0));
        assert!(!withheld_600().record_state_cost(1_301));
    }

    /// A history charge past the reservoir spills the same way.
    #[test]
    fn test_a_history_spill_draws_the_withheld_part_first() {
        let mut tracker = withheld_600();

        assert!(tracker.record_history_cost(500));

        assert_eq!(tracker.reservoir(), 0);
        assert_eq!(parts(&tracker), (400, 400, 800));
        assert_eq!(tracker.state_gas_spilled(), 200);
    }

    /// Rolling a spill back credits the spendable part, whichever part the spill came from.
    #[test]
    fn test_a_rolled_back_spill_lands_on_the_spendable_part() {
        let mut tracker = withheld_600();
        assert!(tracker.record_state_cost(500));

        tracker.rollback_state_gas();

        assert_eq!(tracker.reservoir(), 300);
        assert_eq!(parts(&tracker), (600, 400, 1_000));
    }

    /// Refilling a spill credits the spendable part.
    #[test]
    fn test_refills_land_on_the_spendable_part() {
        let mut tracker = withheld_600();
        assert!(tracker.record_state_cost(500));
        tracker.refill_reservoir(100);
        assert_eq!(parts(&tracker), (500, 400, 900));

        let mut tracker = withheld_600();
        assert!(tracker.record_history_cost(500));
        tracker.refill_history(100);
        assert_eq!(parts(&tracker), (500, 400, 900));

        let mut tracker = withheld_600();
        assert!(tracker.record_state_cost(500));
        tracker.absorb_returned_reservoir(100);
        assert_eq!(parts(&tracker), (500, 400, 900));
    }

    /// The gas a child returns lands on the spendable part.
    #[test]
    fn test_erase_cost_lands_on_the_spendable_part() {
        let mut tracker = withheld_600();
        assert!(tracker.record_withheld_first_cost(700));
        assert_eq!(parts(&tracker), (300, 0, 300));

        tracker.erase_cost(700);

        assert_eq!(parts(&tracker), (1_000, 0, 1_000));
    }

    /// Spending all gas zeroes both parts and keeps the crossing record, so the withheld part
    /// the halt zeroed can still be read from it.
    #[test]
    fn test_spend_all_zeroes_both_parts_and_keeps_the_record() {
        let mut tracker = withheld_600();
        assert!(!tracker.record_regular_cost(401));

        tracker.spend_all();

        assert_eq!(parts(&tracker), (0, 0, 0));
        assert_eq!(tracker.reservoir(), 300);
        assert_eq!(recorded(&tracker), Some(600), "the record survives");
    }

    /// `set_remaining` sets the total. The withheld part is kept as far as the total allows.
    #[test]
    fn test_set_remaining_sets_the_total() {
        let mut tracker = withheld_600();

        tracker.set_remaining(700);
        assert_eq!(parts(&tracker), (100, 600, 700));

        tracker.set_remaining(250);
        assert_eq!(parts(&tracker), (0, 250, 250));

        tracker.set_remaining(900);
        assert_eq!(parts(&tracker), (650, 250, 900));
    }

    /// Encodings made before the withheld part existed decode with nothing withheld.
    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_round_trip_and_older_payloads() {
        let mut tracker = withheld_600();
        assert!(!tracker.record_regular_cost(401));
        let encoded = serde_json::to_value(tracker).unwrap();
        assert_eq!(
            serde_json::from_value::<GasTracker>(encoded.clone()).unwrap(),
            tracker
        );

        let mut older = encoded;
        let fields = older.as_object_mut().unwrap();
        fields.remove("withheld");
        fields.remove("withheld_crossing");
        let decoded = serde_json::from_value::<GasTracker>(older).unwrap();
        assert_eq!(parts(&decoded), (400, 0, 400));
        assert_eq!(decoded.withheld_crossing(), None);
    }
}
