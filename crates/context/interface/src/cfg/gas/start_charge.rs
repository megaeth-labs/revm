//! The start charge a [`Host::frame_starting`](crate::Host::frame_starting) hook declares on the
//! [`FrameStart`](crate::host::FrameStart) it is handed, and the instruction copies onto the
//! frame's input.
//!
//! The hook charges the caller's gas for the frame it starts; the part of that charge the frame's
//! failure gives back is the start charge. The hook declares it on the start
//! ([`FrameStart::charge`](crate::host::FrameStart::charge)), the instruction copies it onto the
//! frame's input when the hook admits the start, and the handler refunds it to the caller when the
//! frame fails.

/// What a frame start charged its caller and the frame's failure gives back: history gas and
/// bytes of the pool.
///
/// Built with [`StartCharge::new`], so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct StartCharge {
    /// History gas the start charged, refilled with
    /// [`refill_history`](super::GasTracker::refill_history) when the frame fails.
    pub history: u64,
    /// Bytes the start drew from the pool, refilled with
    /// [`refill_bytes`](super::GasTracker::refill_bytes) when the frame fails.
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
