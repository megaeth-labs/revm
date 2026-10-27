//! The start lifecycle revm keeps for a transaction, as the [`Host`](super::Host) hooks see it.
//!
//! A start opens where revm consumes a frame's init, just before it initialises the frame
//! ([`Host::frame_split`](super::Host::frame_split)), and closes where revm settles the frame's
//! result into its caller ([`Host::frame_settle`](super::Host::frame_settle)). The transaction's
//! lifecycle ends at one of two boundaries ([`Host::frame_lifecycle_end`](super::Host::frame_lifecycle_end)):
//! the successful one, where every start opened has settled, or the fatal one, where the starts
//! still open are discarded.
//!
//! revm owns the pairing. It allocates each start's identity, keeps the open starts in its frame
//! stack ([`FrameStack`](crate::FrameStack)), which no inspector is handed, and settles each
//! start it opened once, innermost first. The values here are built by that record only.

/// What became of a start revm settles into its caller, read from the frame's result.
///
/// Exhaustive on purpose: a new variant is a breaking change, so that every consumer handles every
/// kind of settlement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettleStatus {
    /// The frame succeeded.
    Ok,
    /// The frame reverted.
    Revert,
    /// The frame halted.
    Halt,
}

/// How a transaction's lifecycle ended.
///
/// Exhaustive on purpose: a new variant is a breaking change, so that every consumer handles every
/// end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecycleEnd {
    /// The successful boundary: every start opened in the transaction settled.
    Settled,
    /// The fatal boundary: an error ended the transaction, and the starts still open are
    /// discarded unsettled.
    Discarded,
}

/// A start revm opened: the frame it makes is the next one revm initialises.
///
/// Built by revm's record only, so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FrameSplit {
    /// The start's identity: a sequence number revm allocated in this transaction, from 1, and
    /// never reused in it.
    pub id: u32,
    /// The identity of the start the calling frame runs for, `None` for the transaction's own
    /// frame.
    pub parent: Option<u32>,
    /// The depth of the frame the start makes, from the frame's init: 0 for the transaction's
    /// frame.
    pub depth: usize,
}

/// A start revm settled into its caller: the caller's gas has absorbed the frame's result.
///
/// Built by revm's record only, so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct FrameSettle {
    /// The identity revm allocated for the start when it opened ([`FrameSplit::id`]).
    pub id: u32,
    /// The depth of the frame the start made.
    pub depth: usize,
    /// What became of the frame.
    pub status: SettleStatus,
    /// For a creation, whether it took an address; `None` for a call.
    pub created: Option<bool>,
    /// An inspector answered the start: its `frame_start`, `call` or `create` returned the
    /// result, so no frame ran.
    pub inspector_answer: bool,
    /// An inspector's end callbacks (`frame_end`, `call_end`, `create_end`) replaced the result's
    /// gas tracker with one revm did not mark for this start.
    pub gas_replaced: bool,
    /// An inspector's end callbacks replaced the result's outcome with one revm did not mark for
    /// this start, such as one a constructor built around the same result.
    pub outcome_rebuilt: bool,
    /// The reservoir the caller's gas held just before revm settled the result into it.
    ///
    /// revm's settlement replaces the caller's reservoir with the one the frame returns, once that
    /// has unwound the caller's outstanding spill. A consumer that kept part of the caller's
    /// reservoir when the start was made ([`FrameStart::reservoir`](super::FrameStart::reservoir))
    /// adds this back to it here, which gives the caller what the frame returned on top of the
    /// part it kept, since the unwinding and every refill read the spill, not the reservoir. The
    /// transaction's frame is handed a copy of the transaction's reservoir, which this then is: a
    /// consumer adds back only what it kept.
    pub held_reservoir: u64,
}
