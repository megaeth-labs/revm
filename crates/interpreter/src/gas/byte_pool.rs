//! The byte pool and the history allowance, on [`Gas`].
//!
//! Each method forwards to the [`GasTracker`](super::GasTracker) method of the same name, which
//! documents it.

use super::Gas;

impl Gas {
    /// The byte pool: what this frame may still draw. Unlimited unless a consumer set a pool.
    ///
    /// See [`GasTracker::bytes_remaining`](super::GasTracker::bytes_remaining).
    #[inline]
    pub const fn bytes_remaining(&self) -> u64 {
        self.tracker.bytes_remaining()
    }

    /// Sets the byte pool.
    ///
    /// See [`GasTracker::set_bytes_remaining`](super::GasTracker::set_bytes_remaining).
    #[inline]
    pub const fn set_bytes_remaining(&mut self, val: u64) {
        self.tracker.set_bytes_remaining(val);
    }

    /// The frame's net draw of the pool so far, signed.
    ///
    /// See [`GasTracker::bytes_spent`](super::GasTracker::bytes_spent).
    #[inline]
    pub const fn bytes_spent(&self) -> i64 {
        self.tracker.bytes_spent()
    }

    /// Sets the net draw.
    ///
    /// See [`GasTracker::set_bytes_spent`](super::GasTracker::set_bytes_spent).
    #[inline]
    pub const fn set_bytes_spent(&mut self, val: i64) {
        self.tracker.set_bytes_spent(val);
    }

    /// Draws `cost` bytes from the pool; `false` if the pool cannot pay, which sets the crossing
    /// ([`bytes_crossing`](Self::bytes_crossing)).
    ///
    /// See [`GasTracker::record_bytes`](super::GasTracker::record_bytes).
    #[inline]
    #[must_use = "a draw the pool cannot pay is the consumer's stop"]
    pub const fn record_bytes(&mut self, cost: u64) -> bool {
        self.tracker.record_bytes(cost)
    }

    /// Gives `amount` bytes back to the pool.
    ///
    /// See [`GasTracker::refill_bytes`](super::GasTracker::refill_bytes).
    #[inline]
    pub const fn refill_bytes(&mut self, amount: u64) {
        self.tracker.refill_bytes(amount);
    }

    /// Whether a draw failed that the pool could not pay, since the record was last cleared.
    ///
    /// See [`GasTracker::bytes_crossing`](super::GasTracker::bytes_crossing).
    #[inline]
    pub const fn bytes_crossing(&self) -> bool {
        self.tracker.bytes_crossing()
    }

    /// Clears the record returned by [`bytes_crossing`](Self::bytes_crossing).
    ///
    /// See [`GasTracker::clear_bytes_crossing`](super::GasTracker::clear_bytes_crossing).
    #[inline]
    pub const fn clear_bytes_crossing(&mut self) {
        self.tracker.clear_bytes_crossing();
    }

    /// Sets the record returned by [`bytes_crossing`](Self::bytes_crossing).
    ///
    /// See [`GasTracker::set_bytes_crossing`](super::GasTracker::set_bytes_crossing).
    #[inline]
    pub const fn set_bytes_crossing(&mut self, crossing: bool) {
        self.tracker.set_bytes_crossing(crossing);
    }

    /// History gas this frame may spend before it charges its budget.
    ///
    /// See [`GasTracker::history_allowance`](super::GasTracker::history_allowance).
    #[inline]
    pub const fn history_allowance(&self) -> u64 {
        self.tracker.history_allowance()
    }

    /// Sets the history allowance.
    ///
    /// See [`GasTracker::set_history_allowance`](super::GasTracker::set_history_allowance).
    #[inline]
    pub const fn set_history_allowance(&mut self, val: u64) {
        self.tracker.set_history_allowance(val);
    }

    /// Draws `min(cost, allowance)` from the allowance and returns the part of `cost` it did not
    /// cover.
    ///
    /// See [`GasTracker::draw_history_allowance`](super::GasTracker::draw_history_allowance).
    #[inline]
    #[must_use = "the part of the cost the allowance did not cover is still to be charged"]
    pub const fn draw_history_allowance(&mut self, cost: u64) -> u64 {
        self.tracker.draw_history_allowance(cost)
    }
}
