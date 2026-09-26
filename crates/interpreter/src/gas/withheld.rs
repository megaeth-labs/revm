//! The withheld part of a frame's regular gas, on [`Gas`].
//!
//! Each method forwards to the [`GasTracker`](super::GasTracker) method of the same name, which
//! documents it.

use super::{Gas, WithheldCrossing};

impl Gas {
    /// Returns the spendable part of the regular gas: what a regular charge may draw.
    ///
    /// See [`GasTracker::spendable`](super::GasTracker::spendable).
    #[inline]
    pub const fn spendable(&self) -> u64 {
        self.tracker.spendable()
    }

    /// Returns the withheld part of the regular gas: gas held back from regular charges but
    /// still counted by [`remaining`](Self::remaining).
    ///
    /// See [`GasTracker::withheld`](super::GasTracker::withheld).
    #[inline]
    pub const fn withheld(&self) -> u64 {
        self.tracker.withheld()
    }

    /// Moves `min(amount, spendable)` from the spendable part to the withheld part, adding it to
    /// what is already withheld and leaving [`remaining`](Self::remaining) unchanged.
    ///
    /// See [`GasTracker::withhold`](super::GasTracker::withhold).
    #[inline]
    pub const fn withhold(&mut self, amount: u64) {
        self.tracker.withhold(amount);
    }

    /// Makes the spendable part `min(allowance, remaining())` and withholds the rest, leaving
    /// [`remaining`](Self::remaining) unchanged. What a consumer aiming at an allowance calls.
    ///
    /// See [`GasTracker::limit_spendable`](super::GasTracker::limit_spendable).
    #[inline]
    pub const fn limit_spendable(&mut self, allowance: u64) {
        self.tracker.limit_spendable(allowance);
    }

    /// Moves the whole withheld part back to the spendable part, leaving
    /// [`remaining`](Self::remaining) unchanged.
    ///
    /// See [`GasTracker::release_withheld`](super::GasTracker::release_withheld).
    #[inline]
    pub const fn release_withheld(&mut self) {
        self.tracker.release_withheld();
    }

    /// Returns the record of the last failed regular charge the withheld part would have paid,
    /// since the record was last cleared: the regular gas left before that charge, spendable and
    /// withheld parts together. It survives [`spend_all`](Self::spend_all).
    ///
    /// See [`GasTracker::withheld_crossing`](super::GasTracker::withheld_crossing).
    #[inline]
    pub const fn withheld_crossing(&self) -> Option<WithheldCrossing> {
        self.tracker.withheld_crossing()
    }

    /// Clears the record returned by [`withheld_crossing`](Self::withheld_crossing).
    ///
    /// See [`GasTracker::clear_withheld_crossing`](super::GasTracker::clear_withheld_crossing).
    #[inline]
    pub const fn clear_withheld_crossing(&mut self) {
        self.tracker.clear_withheld_crossing();
    }

    /// Sets the record returned by [`withheld_crossing`](Self::withheld_crossing), or clears it
    /// with `None`. A consumer marks a result it produced itself, such as a precompile's, as a
    /// crossing, and a classifier then treats it as one a failed charge recorded.
    ///
    /// See [`GasTracker::set_withheld_crossing`](super::GasTracker::set_withheld_crossing).
    #[inline]
    pub const fn set_withheld_crossing(&mut self, crossing: Option<WithheldCrossing>) {
        self.tracker.set_withheld_crossing(crossing);
    }

    /// Records a deduction that is not the frame's own regular work, such as the gas forwarded to
    /// a child frame: draws the withheld part first, then the spendable part, and fails only when
    /// [`remaining`](Self::remaining) cannot pay.
    ///
    /// See [`GasTracker::record_withheld_first_cost`](super::GasTracker::record_withheld_first_cost).
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_withheld_first_cost(&mut self, cost: u64) -> bool {
        self.tracker.record_withheld_first_cost(cost)
    }
}

#[cfg(test)]
mod tests {
    use super::{Gas, WithheldCrossing};
    use core::num::NonZeroU64;

    fn parts(gas: &Gas) -> (u64, u64, u64) {
        (gas.spendable(), gas.withheld(), gas.remaining())
    }

    /// Withheld gas is not spent: every figure derived from the remaining gas is the one the
    /// frame shows with nothing withheld.
    #[test]
    #[allow(deprecated)]
    fn test_withheld_gas_counts_as_unspent() {
        let mut plain = Gas::new(1_000);
        assert!(plain.record_regular_cost(100));
        plain.record_refund(30);
        let mut withheld = plain;

        withheld.withhold(500);

        assert_eq!((withheld.spendable(), withheld.withheld()), (400, 500));
        assert_eq!(withheld.remaining(), plain.remaining());
        assert_eq!(withheld.spent(), plain.spent());
        assert_eq!(withheld.total_gas_spent(), plain.total_gas_spent());
        assert_eq!(withheld.used(), plain.used());
        assert_eq!(withheld.spent_sub_refunded(), plain.spent_sub_refunded());
    }

    /// `set_spent` sets the total through `set_remaining`, keeping the withheld part.
    #[test]
    fn test_set_spent_sets_the_total() {
        let mut gas = Gas::new(1_000);
        gas.withhold(300);

        gas.set_spent(200);

        assert_eq!(gas.total_gas_spent(), 200);
        assert_eq!((gas.spendable(), gas.withheld()), (500, 300));
    }

    /// The step loop's unchecked charge on `Gas` records a crossing, and `spend_all` keeps it.
    #[test]
    fn test_record_cost_unsafe_records_a_crossing() {
        let mut gas = Gas::new(100);
        gas.withhold(90);

        assert!(gas.record_cost_unsafe(11));
        gas.spend_all();

        assert_eq!(
            (gas.spendable(), gas.withheld(), gas.remaining()),
            (0, 0, 0)
        );
        let crossing = gas.withheld_crossing().expect("a crossing");
        assert_eq!(crossing.remaining(), 100, "the regular gas the halt zeroed");

        gas.set_remaining(crossing.remaining());
        assert_eq!(gas.remaining(), 100, "put back exactly");

        gas.clear_withheld_crossing();
        assert_eq!(gas.withheld_crossing(), None);
    }

    /// A record a consumer sets reads back as the record a failed charge leaves with the same
    /// regular gas left, leaves both parts alone, and is cleared by setting `None`.
    #[test]
    fn test_set_withheld_crossing_sets_reads_and_clears() {
        let mut charged = Gas::new(100);
        charged.withhold(90);
        assert!(!charged.record_regular_cost(11));

        let mut marked = Gas::new(100);
        marked.withhold(90);
        let crossing = WithheldCrossing::with_remaining(NonZeroU64::new(100).unwrap());

        marked.set_withheld_crossing(Some(crossing));

        assert_eq!(marked.withheld_crossing(), Some(crossing));
        assert_eq!(marked.withheld_crossing(), charged.withheld_crossing());
        assert_eq!(parts(&marked), (10, 90, 100), "the parts are untouched");

        marked.set_withheld_crossing(None);
        assert_eq!(marked.withheld_crossing(), None);
        assert_eq!(parts(&marked), (10, 90, 100));
    }

    /// A record a consumer sets survives `spend_all`, as a recorded one does.
    #[test]
    fn test_a_set_crossing_survives_spend_all() {
        let mut gas = Gas::new(100);
        gas.withhold(90);
        let crossing = WithheldCrossing::with_remaining(NonZeroU64::new(100).unwrap());
        gas.set_withheld_crossing(Some(crossing));

        gas.spend_all();

        assert_eq!(parts(&gas), (0, 0, 0));
        assert_eq!(gas.withheld_crossing(), Some(crossing));
    }

    /// A frame limited to an allowance, then forwarding part of its withheld gas, keeps the
    /// allowance when limited again: the next regular charge succeeds.
    #[test]
    fn test_limit_spendable_keeps_the_allowance_across_a_forward() {
        let mut gas = Gas::new(100_000);
        gas.limit_spendable(10_000);
        assert_eq!((gas.spendable(), gas.withheld()), (10_000, 90_000));

        assert!(gas.record_withheld_first_cost(50_000));
        assert_eq!((gas.spendable(), gas.withheld()), (10_000, 40_000));
        gas.limit_spendable(10_000);
        assert_eq!((gas.spendable(), gas.withheld()), (10_000, 40_000));

        assert!(gas.record_regular_cost(3));
        assert_eq!(gas.withheld_crossing(), None);
        assert_eq!((gas.spendable(), gas.withheld()), (9_997, 40_000));
    }

    /// Releasing and forwarding go through to the tracker.
    #[test]
    fn test_release_and_forward() {
        let mut gas = Gas::new(100);
        gas.withhold(90);
        assert!(gas.record_withheld_first_cost(50));
        assert_eq!((gas.spendable(), gas.withheld()), (10, 40));

        gas.release_withheld();

        assert_eq!(
            (gas.spendable(), gas.withheld(), gas.remaining()),
            (50, 0, 50)
        );
    }
}
