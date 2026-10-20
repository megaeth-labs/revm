//! The record a failed regular charge leaves, on [`Gas`].
//!
//! The method forwards to the [`GasTracker`](super::GasTracker) method of the same name, which
//! documents it.

use super::Gas;

impl Gas {
    /// Returns the regular gas left, spendable and withheld parts together, before the last
    /// regular charge that failed, or `None` if none failed. It survives
    /// [`spend_all`](Self::spend_all), so it can be read from the result of a frame that halted.
    ///
    /// See [`GasTracker::failed_charge_remaining`](super::GasTracker::failed_charge_remaining).
    #[inline]
    pub const fn failed_charge_remaining(&self) -> Option<u64> {
        self.tracker.failed_charge_remaining()
    }
}
