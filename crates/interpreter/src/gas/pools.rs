//! The resource pools and the history allowance, on [`Gas`].
//!
//! Each method forwards to the [`GasTracker`](super::GasTracker) method of the same name, which
//! documents it.

use super::{Gas, PoolId};

impl Gas {
    /// The pool: what this frame may still draw from it. Unlimited unless a consumer set it.
    ///
    /// See [`GasTracker::pool_remaining`](super::GasTracker::pool_remaining).
    #[inline]
    pub const fn pool_remaining(&self, pool: PoolId) -> u64 {
        self.tracker.pool_remaining(pool)
    }

    /// Sets the pool.
    ///
    /// See [`GasTracker::set_pool_remaining`](super::GasTracker::set_pool_remaining).
    #[inline]
    pub const fn set_pool_remaining(&mut self, pool: PoolId, val: u64) {
        self.tracker.set_pool_remaining(pool, val);
    }

    /// The frame's net draw of the pool so far, signed.
    ///
    /// See [`GasTracker::pool_spent`](super::GasTracker::pool_spent).
    #[inline]
    pub const fn pool_spent(&self, pool: PoolId) -> i64 {
        self.tracker.pool_spent(pool)
    }

    /// Sets the pool's net draw.
    ///
    /// See [`GasTracker::set_pool_spent`](super::GasTracker::set_pool_spent).
    #[inline]
    pub const fn set_pool_spent(&mut self, pool: PoolId, val: i64) {
        self.tracker.set_pool_spent(pool, val);
    }

    /// Draws `cost` from the pool; `false` if the pool cannot pay, which sets its crossing
    /// ([`pool_crossing`](Self::pool_crossing)).
    ///
    /// See [`GasTracker::record_pool`](super::GasTracker::record_pool).
    #[inline]
    #[must_use = "a draw the pool cannot pay is the consumer's stop"]
    pub const fn record_pool(&mut self, pool: PoolId, cost: u64) -> bool {
        self.tracker.record_pool(pool, cost)
    }

    /// Gives `amount` back to the pool.
    ///
    /// See [`GasTracker::refill_pool`](super::GasTracker::refill_pool).
    #[inline]
    pub const fn refill_pool(&mut self, pool: PoolId, amount: u64) {
        self.tracker.refill_pool(pool, amount);
    }

    /// Whether a draw failed that the pool could not pay, since the record was last cleared.
    ///
    /// See [`GasTracker::pool_crossing`](super::GasTracker::pool_crossing).
    #[inline]
    pub const fn pool_crossing(&self, pool: PoolId) -> bool {
        self.tracker.pool_crossing(pool)
    }

    /// Clears the record returned by [`pool_crossing`](Self::pool_crossing).
    ///
    /// See [`GasTracker::clear_pool_crossing`](super::GasTracker::clear_pool_crossing).
    #[inline]
    pub const fn clear_pool_crossing(&mut self, pool: PoolId) {
        self.tracker.clear_pool_crossing(pool);
    }

    /// Sets the record returned by [`pool_crossing`](Self::pool_crossing).
    ///
    /// See [`GasTracker::set_pool_crossing`](super::GasTracker::set_pool_crossing).
    #[inline]
    pub const fn set_pool_crossing(&mut self, pool: PoolId, crossing: bool) {
        self.tracker.set_pool_crossing(pool, crossing);
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
