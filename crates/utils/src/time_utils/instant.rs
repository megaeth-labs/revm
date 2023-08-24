//! This Instant measures time with high performance and high accuracy powered by TSC.
use super::cycles;
use std::time::Duration;

/// A measurement of a monotonically nondecreasing clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct Instant(u64);

impl Instant {
    /// Returns an instant corresponding to "now".
    #[inline]
    pub fn now() -> Instant {
        Instant(cycles::rdtsc())
    }

    /// Returns the amount of cpu cycles from another instant to this one,
    /// or None if that instant is later than this one.
    pub fn checked_cycles_since(&self, earlier: Instant) -> Option<u64> {
        Some(self.0.checked_sub(earlier.0)?)
    }

    /// Returns the amount of nanos from another instant to this one,
    /// or None if that instant is later than this one.
    pub fn checked_nanos_since(&self, earlier: Instant) -> Option<f64> {
        Some(cycles::convert_cycles_to_ns_f64(
            self.0.checked_sub(earlier.0)?,
        ))
    }

    /// Returns the amount of duration from another instant to this one,
    /// or None if that instant is later than this one.
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        Some(cycles::convert_cycles_to_duration(
            self.0.checked_sub(earlier.0)?,
        ))
    }
}
