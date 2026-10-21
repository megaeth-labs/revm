//! The start charge a [`Host::frame_starting`](crate::Host::frame_starting) hook declares on the
//! caller's [`GasTracker`], and the instruction copies onto the frame's input.
//!
//! The hook charges the caller's gas for the frame it starts; the part of that charge the frame's
//! failure gives back is the start charge. The hook declares it on the tracker it is handed
//! ([`set_start_charge`](GasTracker::set_start_charge)), the instruction takes it off the tracker
//! right after the hook ([`take_start_charge`](GasTracker::take_start_charge)) and puts it on the
//! frame's input, and the handler refunds it to the caller when the frame fails. Outside that
//! window the carrier is zero.

use super::GasTracker;

/// What a frame start charged its caller and the frame's failure gives back: history gas and
/// bytes of the pool.
///
/// Built with [`StartCharge::new`], so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct StartCharge {
    /// History gas the start charged, refilled with
    /// [`refill_history`](GasTracker::refill_history) when the frame fails.
    pub history: u64,
    /// Bytes the start drew from the pool, refilled with
    /// [`refill_bytes`](GasTracker::refill_bytes) when the frame fails.
    pub bytes: u64,
}

impl StartCharge {
    /// A start charge of `history` gas and `bytes` of the pool.
    #[inline]
    pub const fn new(history: u64, bytes: u64) -> Self {
        Self { history, bytes }
    }

    /// Nothing to refund.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.history == 0 && self.bytes == 0
    }
}

impl GasTracker {
    /// The start charge the frame-start hook declared and the instruction has not taken yet.
    #[inline]
    pub const fn start_charge(&self) -> StartCharge {
        self.start_charge
    }

    /// Declares the start charge of the frame about to start: what the hook charged this tracker
    /// that the frame's failure gives back. Replaces what was declared before.
    #[inline]
    pub const fn set_start_charge(&mut self, charge: StartCharge) {
        self.start_charge = charge;
    }

    /// Takes the declared start charge off the tracker, leaving none.
    #[inline]
    pub const fn take_start_charge(&mut self) -> StartCharge {
        let charge = self.start_charge;
        self.start_charge = StartCharge::new(0, 0);
        charge
    }
}

#[cfg(test)]
mod tests {
    use super::{GasTracker, StartCharge};

    /// A tracker carries no start charge until a hook declares one; taking it leaves none.
    #[test]
    fn test_the_carrier_is_empty_until_set_and_after_take() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        assert!(tracker.start_charge().is_zero());
        assert_eq!(tracker.take_start_charge(), StartCharge::default());

        tracker.set_start_charge(StartCharge::new(3_520, 160));
        assert_eq!(tracker.start_charge(), StartCharge::new(3_520, 160));

        assert_eq!(tracker.take_start_charge(), StartCharge::new(3_520, 160));
        assert!(tracker.start_charge().is_zero());
        assert_eq!(tracker, GasTracker::new(1_000, 1_000, 0));
    }

    /// A second declaration replaces the first.
    #[test]
    fn test_a_second_declaration_replaces_the_first() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        tracker.set_start_charge(StartCharge::new(1, 1));
        tracker.set_start_charge(StartCharge::new(2, 0));
        assert_eq!(tracker.take_start_charge(), StartCharge::new(2, 0));
    }

    /// The carrier is transient and not part of an encoded tracker.
    #[cfg(feature = "serde")]
    #[test]
    fn test_the_carrier_is_not_encoded() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        tracker.set_start_charge(StartCharge::new(5, 5));
        let encoded = serde_json::to_value(tracker).unwrap();
        assert!(encoded.get("start_charge").is_none());
        let decoded = serde_json::from_value::<GasTracker>(encoded).unwrap();
        assert!(decoded.start_charge().is_zero());
    }
}
