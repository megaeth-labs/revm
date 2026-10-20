//! The record a failed charge leaves, on [`Gas`].
//!
//! The first two methods forward to the [`GasTracker`](super::GasTracker) methods of the same
//! name, which document them. The third is how the interpreter records an out-of-gas halt that no
//! failed deduction recorded.

use super::Gas;

impl Gas {
    /// Returns the regular gas left, spendable and withheld parts together, before the last
    /// charge that failed, or `None` if none failed. It survives [`spend_all`](Self::spend_all),
    /// so it can be read from the result of a frame that halted.
    ///
    /// See [`GasTracker::failed_charge_remaining`](super::GasTracker::failed_charge_remaining).
    #[inline]
    pub const fn failed_charge_remaining(&self) -> Option<u64> {
        self.tracker.failed_charge_remaining()
    }

    /// Sets the record returned by [`failed_charge_remaining`](Self::failed_charge_remaining), or
    /// clears it with `None`, for a result produced outside the tracker that ends a frame out of
    /// gas.
    ///
    /// See
    /// [`GasTracker::set_failed_charge_remaining`](super::GasTracker::set_failed_charge_remaining).
    #[inline]
    pub const fn set_failed_charge_remaining(&mut self, remaining: Option<u64>) {
        self.tracker.set_failed_charge_remaining(remaining);
    }

    /// Records the regular gas left, unless a record is already there: what an out-of-gas halt of
    /// the interpreter is about to zero.
    ///
    /// Called by [`Interpreter::halt`](crate::Interpreter::halt) and
    /// [`Interpreter::halt_oog`](crate::Interpreter::halt_oog), before they zero the regular gas.
    /// Every failed deduction records itself and halts the frame, so a record already there is the
    /// failure that caused this halt, and it is kept; the unchecked charge's record also holds
    /// the regular gas from before the spendable part wrapped. With none there, nothing failed:
    /// the halt is a cold access the instruction refused before its charge, because the frame
    /// could not pay it, or an out-of-gas a consumer's instruction returned itself. Either way the
    /// regular gas is as it was when the frame stopped.
    #[inline]
    pub(crate) const fn record_out_of_gas_halt(&mut self) {
        if self.failed_charge_remaining().is_none() {
            self.set_failed_charge_remaining(Some(self.remaining()));
        }
    }
}
