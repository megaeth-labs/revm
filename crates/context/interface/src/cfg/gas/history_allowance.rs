//! The history allowance of a [`GasTracker`]: history gas a frame may spend before it charges
//! its budget.
//!
//! The fork sets it on a value call's child, from the schedule's `call_history_allowance`, where
//! it adds the call stipend; it is zero on every schedule defined here and in every other frame.
//! Nothing in this workspace draws it: a consumer that charges history for a write draws the
//! allowance first ([`draw_history_allowance`](GasTracker::draw_history_allowance)) and charges
//! the rest. The allowance is the frame's alone: it is not returned, merged or rolled back.

use super::GasTracker;

impl GasTracker {
    /// History gas this frame may spend before it charges its budget.
    #[inline]
    pub const fn history_allowance(&self) -> u64 {
        self.history_allowance
    }

    /// Sets the history allowance.
    #[inline]
    pub const fn set_history_allowance(&mut self, val: u64) {
        self.history_allowance = val;
    }

    /// Draws `min(cost, allowance)` from the allowance and returns the part of `cost` the
    /// allowance did not cover, which the caller charges as history gas.
    ///
    /// With no allowance, which is the default, it returns `cost` whole and changes nothing.
    #[inline]
    #[must_use = "the part of the cost the allowance did not cover is still to be charged"]
    pub const fn draw_history_allowance(&mut self, cost: u64) -> u64 {
        let drawn = if cost < self.history_allowance {
            cost
        } else {
            self.history_allowance
        };
        self.history_allowance -= drawn;
        cost - drawn
    }
}

#[cfg(test)]
mod tests {
    use super::GasTracker;

    /// Every tracker starts with no allowance, and a draw on none is the whole cost.
    #[test]
    fn test_no_allowance_leaves_the_whole_cost_to_charge() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        assert_eq!(tracker.history_allowance(), 0);
        assert_eq!(tracker.draw_history_allowance(100), 100);
        assert_eq!(tracker, GasTracker::new(1_000, 1_000, 0));
    }

    /// A draw within the allowance leaves nothing to charge; one beyond it empties the allowance
    /// and leaves the rest.
    #[test]
    fn test_draws_take_the_allowance_first() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        tracker.set_history_allowance(160);

        assert_eq!(tracker.draw_history_allowance(100), 0);
        assert_eq!(tracker.history_allowance(), 60);

        assert_eq!(tracker.draw_history_allowance(100), 40);
        assert_eq!(tracker.history_allowance(), 0);

        assert_eq!(tracker.draw_history_allowance(0), 0);
    }

    /// The allowance touches no other figure of the tracker.
    #[test]
    fn test_the_allowance_is_its_own_figure() {
        let mut tracker = GasTracker::new(1_000, 1_000, 300);
        tracker.set_history_allowance(160);
        assert_eq!(tracker.draw_history_allowance(50), 0);
        assert_eq!(tracker.remaining(), 1_000);
        assert_eq!(tracker.reservoir(), 300);
        assert_eq!(tracker.history_gas_spent(), 0);
    }

    /// Encodings made before the allowance existed decode with none.
    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_round_trip_and_older_payloads() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        tracker.set_history_allowance(160);
        let encoded = serde_json::to_value(tracker).unwrap();
        assert_eq!(
            serde_json::from_value::<GasTracker>(encoded.clone()).unwrap(),
            tracker
        );

        let mut older = encoded;
        older.as_object_mut().unwrap().remove("history_allowance");
        let decoded = serde_json::from_value::<GasTracker>(older).unwrap();
        assert_eq!(decoded.history_allowance(), 0);
    }
}
