//! The start lifecycle revm keeps in its [`FrameStack`]: the open start of the transaction and
//! of each frame on the stack, with the sequence their identities come from.
//!
//! The record decides nothing a consumer computes: it pairs every settlement with the start it
//! closes, and refuses every step that would break the pairing ([`StartLifecycleError`]).

use super::FrameStack;
use crate::host::{FrameSettle, FrameSplit, LifecycleEnd, SettleStatus};
use std::vec::Vec;

/// A start revm opened and has not settled, with what an inspector did to it so far.
///
/// Its identity is the record's: it can be read, not changed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PendingStart {
    id: u32,
    inspector_answer: bool,
    gas_replaced: bool,
    outcome_rebuilt: bool,
}

impl PendingStart {
    /// The start's identity ([`FrameSplit::id`]).
    #[inline]
    pub const fn id(&self) -> u32 {
        self.id
    }

    /// Whether an inspector answered the start ([`FrameSettle::inspector_answer`]).
    #[inline]
    pub const fn inspector_answer(&self) -> bool {
        self.inspector_answer
    }

    /// Whether an inspector's end callbacks replaced the result's gas
    /// ([`FrameSettle::gas_replaced`]).
    #[inline]
    pub const fn gas_replaced(&self) -> bool {
        self.gas_replaced
    }

    /// Whether an inspector's end callbacks rebuilt the result's outcome
    /// ([`FrameSettle::outcome_rebuilt`]).
    #[inline]
    pub const fn outcome_rebuilt(&self) -> bool {
        self.outcome_rebuilt
    }

    /// Records that an inspector answered the start.
    #[inline]
    pub const fn set_inspector_answer(&mut self) {
        self.inspector_answer = true;
    }

    /// Records that an inspector's end callbacks replaced the result's gas.
    #[inline]
    pub const fn set_gas_replaced(&mut self) {
        self.gas_replaced = true;
    }

    /// Records that an inspector's end callbacks rebuilt the result's outcome.
    #[inline]
    pub const fn set_outcome_rebuilt(&mut self) {
        self.outcome_rebuilt = true;
    }
}

/// Why the start lifecycle refused a step. Each is a fatal error of the transaction: the step
/// would have paired a settlement with the wrong start, or with none.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StartLifecycleError {
    /// The transaction's sequence of identities is exhausted: it has opened `u32::MAX` starts.
    Exhausted,
    /// A start opened where one is still open: a settlement or a boundary was missed, such as an
    /// error dropped without the cleanup that discards the lifecycle.
    Stale,
    /// A settlement found no open start: the start was never opened, or already settled.
    NotOpen,
    /// A child start opened under a frame that has no start of its own: a runner drove the
    /// transaction's frame without opening the transaction's start first.
    NoParent,
    /// The successful boundary found a start still open: a settlement was missed.
    Unsettled,
}

impl core::fmt::Display for StartLifecycleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Exhausted => "start lifecycle: the transaction's identities are exhausted",
            Self::Stale => "start lifecycle: a start opened where one is still open",
            Self::NotOpen => "start lifecycle: a settlement found no open start",
            Self::NoParent => {
                "start lifecycle: a child start opened under a frame with no start (open the \
                 transaction's start first)"
            }
            Self::Unsettled => "start lifecycle: the transaction ended with a start open",
        })
    }
}

impl core::error::Error for StartLifecycleError {}

/// The record: the transaction's start and the open start of each frame on the stack, the child
/// it is making.
#[derive(Clone, Debug, Default)]
pub(super) struct StartLifecycle {
    /// The last identity allocated in the transaction, 0 before the first. The first start opened
    /// is always the transaction's, so a lifecycle is under way exactly when this is not 0.
    last: u32,
    /// How many of `children`'s slots hold an open start.
    open_children: u32,
    /// The transaction's start.
    root: Option<PendingStart>,
    /// The open start of the frame at each stack index. Its length grows with the frame stack's
    /// capacity and never shrinks, so frames and transactions reuse the slots without resizing.
    children: Vec<Option<PendingStart>>,
}

impl StartLifecycle {
    #[inline]
    const fn active(&self) -> bool {
        self.last != 0
    }

    /// The slot the next start opens in, and the next settlement closes: the transaction's on an
    /// empty stack, else that of the frame on top, at `index`.
    #[inline]
    fn slot(&mut self, index: Option<usize>) -> Option<&mut Option<PendingStart>> {
        match index {
            None => Some(&mut self.root),
            Some(i) => self.children.get_mut(i),
        }
    }

    #[inline]
    const fn any_open(&self) -> bool {
        self.root.is_some() || self.open_children != 0
    }

    #[inline]
    fn reset(&mut self) {
        self.last = 0;
        self.root = None;
        if self.open_children != 0 {
            self.discard_children();
        }
    }

    /// Empties the children's slots, which only a discarded lifecycle leaves open.
    #[cold]
    #[inline(never)]
    fn discard_children(&mut self) {
        self.children.fill(None);
        self.open_children = 0;
    }

    /// Gives the record a slot for every frame the stack has room for, `len`: the first child
    /// start, and the first after the frame stack grew.
    #[cold]
    #[inline(never)]
    fn grow_children(&mut self, len: usize) {
        self.children.resize(len, None);
    }
}

impl<T> FrameStack<T> {
    /// Opens the start of the frame the next init makes, at `depth`: the child of the frame on
    /// top of the stack, or the transaction's start when the stack is empty. Returns the split to
    /// hand to [`Host::frame_split`](crate::Host::frame_split).
    ///
    /// A frame has at most one open start, and so has the transaction. A failed open changes
    /// nothing.
    ///
    /// # Errors
    ///
    /// [`StartLifecycleError::Exhausted`] once the transaction opened `u32::MAX` starts,
    /// [`StartLifecycleError::Stale`] when the slot already holds an open start, and
    /// [`StartLifecycleError::NoParent`] when the frame on top has no start of its own.
    #[inline]
    pub fn open_start(&mut self, depth: usize) -> Result<FrameSplit, StartLifecycleError> {
        let capacity = self.stack.capacity();
        let record = &mut self.lifecycle;
        let id = record
            .last
            .checked_add(1)
            .ok_or(StartLifecycleError::Exhausted)?;
        let start = Some(PendingStart {
            id,
            ..Default::default()
        });
        let parent = match self.index {
            None => {
                if record.root.is_some() {
                    return Err(StartLifecycleError::Stale);
                }
                record.root = start;
                None
            }
            Some(i) => {
                let parent = match i {
                    0 => record.root,
                    _ => record.children.get(i - 1).copied().flatten(),
                };
                let Some(parent) = parent else {
                    return Err(StartLifecycleError::NoParent);
                };
                if i >= record.children.len() {
                    record.grow_children(capacity.max(i + 1));
                }
                let slot = &mut record.children[i];
                if slot.is_some() {
                    return Err(StartLifecycleError::Stale);
                }
                *slot = start;
                record.open_children += 1;
                Some(parent.id)
            }
        };
        record.last = id;
        Ok(FrameSplit { id, parent, depth })
    }

    /// Closes the start the next settlement closes ([`pending_start`](Self::pending_start)), that
    /// of a frame at `depth` which settled with `status`, into a caller whose gas held
    /// `held_reservoir` just before. Returns the settlement to hand to
    /// [`Host::frame_settle`](crate::Host::frame_settle), with what the start's record says an
    /// inspector did to it.
    ///
    /// # Errors
    ///
    /// [`StartLifecycleError::NotOpen`] when no start is open there.
    #[inline]
    pub fn close_start(
        &mut self,
        depth: usize,
        status: SettleStatus,
        created: Option<bool>,
        held_reservoir: u64,
    ) -> Result<FrameSettle, StartLifecycleError> {
        let record = &mut self.lifecycle;
        let start = match self.index {
            None => record.root.take(),
            Some(i) => {
                let start = record.children.get_mut(i).and_then(Option::take);
                if start.is_some() {
                    record.open_children -= 1;
                }
                start
            }
        }
        .ok_or(StartLifecycleError::NotOpen)?;
        Ok(FrameSettle {
            id: start.id,
            depth,
            status,
            created,
            inspector_answer: start.inspector_answer,
            gas_replaced: start.gas_replaced,
            outcome_rebuilt: start.outcome_rebuilt,
            held_reservoir,
        })
    }

    /// The start the next settlement closes: the open start of the frame on top of the stack, the
    /// child it is making, or the transaction's start when the stack is empty.
    #[inline]
    pub fn pending_start(&mut self) -> Option<&mut PendingStart> {
        self.lifecycle.slot(self.index).and_then(Option::as_mut)
    }

    /// The start the frame on top of the stack runs for: the open start of the frame below it, or
    /// the transaction's start for the frame at the bottom.
    #[inline]
    pub fn running_start(&mut self) -> Option<&mut PendingStart> {
        match self.index? {
            0 => self.lifecycle.root.as_mut(),
            i => self
                .lifecycle
                .children
                .get_mut(i - 1)
                .and_then(Option::as_mut),
        }
    }

    /// Whether a transaction's lifecycle is under way: its first start opened, and neither
    /// boundary ended it.
    #[inline]
    pub const fn lifecycle_active(&self) -> bool {
        self.lifecycle.active()
    }

    /// Whether every start opened in the transaction has settled.
    #[inline]
    pub const fn lifecycle_settled(&self) -> bool {
        !self.lifecycle.any_open()
    }

    /// The successful boundary: ends the lifecycle and resets the record for the next
    /// transaction. Returns the end to hand to
    /// [`Host::frame_lifecycle_end`](crate::Host::frame_lifecycle_end),
    /// [`LifecycleEnd::Settled`], or `None` when no lifecycle was under way.
    ///
    /// # Errors
    ///
    /// [`StartLifecycleError::Unsettled`] when a start is still open. The record is left as it
    /// is, for [`discard_lifecycle`](Self::discard_lifecycle).
    #[inline]
    pub fn settle_lifecycle(&mut self) -> Result<Option<LifecycleEnd>, StartLifecycleError> {
        if self.lifecycle.any_open() {
            return Err(StartLifecycleError::Unsettled);
        }
        let end = self.lifecycle.active().then_some(LifecycleEnd::Settled);
        self.lifecycle.reset();
        Ok(end)
    }

    /// The fatal boundary: discards the starts still open, ends the lifecycle and resets the
    /// record for the next transaction. Returns the end to hand to
    /// [`Host::frame_lifecycle_end`](crate::Host::frame_lifecycle_end),
    /// [`LifecycleEnd::Discarded`], or `None` when no lifecycle was under way, so a second discard
    /// owes nothing.
    ///
    /// It resets the lifecycle only: the frame stack's index is [`clear`](Self::clear)'s.
    #[inline]
    #[must_use = "a lifecycle that was under way owes the consumer its end"]
    pub fn discard_lifecycle(&mut self) -> Option<LifecycleEnd> {
        let end = self.lifecycle.active().then_some(LifecycleEnd::Discarded);
        self.lifecycle.reset();
        end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stack with frames at indices `0..frames`, as revm's loop builds it.
    fn stack(frames: usize) -> FrameStack<u8> {
        let mut stack = FrameStack::new_prealloc(frames.max(1));
        if frames > 0 {
            let token = stack.start_init().consume();
            unsafe { stack.end_init(token) };
            for _ in 1..frames {
                let token = stack.get_next().consume();
                unsafe { stack.push(token) };
            }
        }
        stack
    }

    fn split(id: u32, parent: Option<u32>, depth: usize) -> FrameSplit {
        FrameSplit { id, parent, depth }
    }

    /// Opens the transaction's start and one child per frame, as a nested run does.
    fn open_nested(stack: &mut FrameStack<u8>, frames: usize) {
        stack.clear();
        stack.open_start(0).unwrap();
        for i in 0..frames {
            stack.index = Some(i);
            stack.open_start(i + 1).unwrap();
        }
    }

    #[test]
    fn test_a_nested_run_opens_one_start_per_frame_under_its_parent() {
        let mut stack = stack(2);
        stack.clear();
        assert_eq!(stack.open_start(0), Ok(split(1, None, 0)));
        stack.index = Some(0);
        assert_eq!(stack.open_start(1), Ok(split(2, Some(1), 1)));
        stack.index = Some(1);
        assert_eq!(stack.open_start(2), Ok(split(3, Some(2), 2)));
        assert!(stack.lifecycle_active());
        assert!(!stack.lifecycle_settled());
        assert_eq!(stack.running_start().map(|s| s.id()), Some(2));
        assert_eq!(stack.pending_start().map(|s| s.id()), Some(3));
    }

    #[test]
    fn test_each_settlement_closes_the_start_of_its_slot_innermost_first() {
        let mut stack = stack(2);
        open_nested(&mut stack, 2);
        let settled = stack.close_start(2, SettleStatus::Revert, None, 7).unwrap();
        assert_eq!(
            (settled.id, settled.depth, settled.status),
            (3, 2, SettleStatus::Revert)
        );
        assert_eq!(settled.held_reservoir, 7);
        stack.index = Some(0);
        let settled = stack
            .close_start(1, SettleStatus::Ok, Some(true), 0)
            .unwrap();
        assert_eq!((settled.id, settled.created), (2, Some(true)));
        stack.clear();
        assert_eq!(
            stack
                .close_start(0, SettleStatus::Halt, None, 0)
                .map(|s| s.id),
            Ok(1)
        );
        assert!(stack.lifecycle_settled());
        assert!(
            stack.lifecycle_active(),
            "only a boundary ends the lifecycle"
        );
    }

    #[test]
    fn test_identities_are_never_reused_within_a_transaction() {
        let mut stack = stack(1);
        open_nested(&mut stack, 1);
        stack.close_start(1, SettleStatus::Ok, None, 0).unwrap();
        assert_eq!(
            stack.open_start(1).map(|s| s.id),
            Ok(3),
            "the frame's second child"
        );
        stack.close_start(1, SettleStatus::Ok, None, 0).unwrap();
        stack.clear();
        stack.close_start(0, SettleStatus::Ok, None, 0).unwrap();
        assert_eq!(stack.settle_lifecycle(), Ok(Some(LifecycleEnd::Settled)));
        assert_eq!(
            stack.open_start(0).map(|s| s.id),
            Ok(1),
            "the next transaction restarts"
        );
    }

    #[test]
    fn test_a_second_start_in_an_occupied_slot_is_stale() {
        let mut stack = stack(1);
        open_nested(&mut stack, 1);
        assert_eq!(stack.open_start(1), Err(StartLifecycleError::Stale));
        stack.clear();
        assert_eq!(stack.open_start(0), Err(StartLifecycleError::Stale));
        stack.index = Some(0);
        assert_eq!(
            stack.pending_start().map(|s| s.id()),
            Some(2),
            "nothing changed"
        );
    }

    #[test]
    fn test_a_child_under_a_frame_with_no_start_has_no_parent() {
        let mut stack = stack(2);
        assert_eq!(stack.open_start(1), Err(StartLifecycleError::NoParent));
        stack.clear();
        stack.open_start(0).unwrap();
        stack.index = Some(1);
        assert_eq!(stack.open_start(2), Err(StartLifecycleError::NoParent));
        assert!(stack.pending_start().is_none(), "nothing opened");
    }

    #[test]
    fn test_a_settlement_with_nothing_open_is_refused() {
        let mut stack = stack(1);
        stack.clear();
        assert_eq!(
            stack.close_start(0, SettleStatus::Ok, None, 0),
            Err(StartLifecycleError::NotOpen)
        );
        stack.open_start(0).unwrap();
        stack.close_start(0, SettleStatus::Ok, None, 0).unwrap();
        assert_eq!(
            stack.close_start(0, SettleStatus::Ok, None, 0),
            Err(StartLifecycleError::NotOpen),
            "a start settles once"
        );
        stack.index = Some(0);
        assert_eq!(
            stack.close_start(1, SettleStatus::Ok, None, 0),
            Err(StartLifecycleError::NotOpen),
            "a frame that made no child"
        );
    }

    #[test]
    fn test_an_exhausted_sequence_opens_nothing() {
        let mut stack = stack(1);
        open_nested(&mut stack, 0);
        stack.lifecycle.last = u32::MAX - 1;
        stack.index = Some(0);
        assert_eq!(
            stack.open_start(1).map(|s| s.id),
            Ok(u32::MAX),
            "the last identity"
        );
        stack.close_start(1, SettleStatus::Ok, None, 0).unwrap();
        assert_eq!(stack.open_start(1), Err(StartLifecycleError::Exhausted));
        assert!(stack.pending_start().is_none(), "nothing opened");
        assert_eq!(stack.discard_lifecycle(), Some(LifecycleEnd::Discarded));
        stack.clear();
        assert_eq!(
            stack.open_start(0).map(|s| s.id),
            Ok(1),
            "the next transaction restarts"
        );
    }

    #[test]
    fn test_the_successful_boundary_needs_every_start_settled() {
        let mut stack = stack(1);
        open_nested(&mut stack, 1);
        stack.close_start(1, SettleStatus::Ok, None, 0).unwrap();
        stack.clear();
        assert_eq!(
            stack.settle_lifecycle(),
            Err(StartLifecycleError::Unsettled)
        );
        assert!(
            stack.lifecycle_active(),
            "the record is left for the discard"
        );
        assert_eq!(stack.pending_start().map(|s| s.id()), Some(1));
        stack.close_start(0, SettleStatus::Ok, None, 0).unwrap();
        assert_eq!(stack.settle_lifecycle(), Ok(Some(LifecycleEnd::Settled)));
        assert!(!stack.lifecycle_active());
        assert_eq!(
            stack.settle_lifecycle(),
            Ok(None),
            "no lifecycle is under way"
        );
    }

    #[test]
    fn test_the_fatal_boundary_discards_once() {
        let mut stack = stack(2);
        open_nested(&mut stack, 2);
        assert_eq!(stack.discard_lifecycle(), Some(LifecycleEnd::Discarded));
        assert_eq!(
            stack.discard_lifecycle(),
            None,
            "a second discard owes nothing"
        );
        assert!(stack.lifecycle_settled() && !stack.lifecycle_active());
        assert_eq!(stack.index(), Some(1), "the frame stack is clear's");
        stack.clear();
        assert_eq!(stack.open_start(0).map(|s| s.id), Ok(1));
    }

    /// The children's slots are sized with the frame stack and outlive the transaction: no
    /// boundary shrinks them, and a discard empties them.
    #[test]
    fn test_the_slots_outlive_the_transaction() {
        // Two frames on a stack with room for eight.
        let mut stack = FrameStack::<u8>::new_prealloc(8);
        let token = stack.start_init().consume();
        unsafe { stack.end_init(token) };
        let token = stack.get_next().consume();
        unsafe { stack.push(token) };
        open_nested(&mut stack, 2);
        let slots = stack.lifecycle.children.len();
        assert!(
            slots >= stack.stack.capacity(),
            "the first child gives a slot to every frame the stack has room for: {slots} of {}",
            stack.stack.capacity()
        );
        assert_eq!(stack.discard_lifecycle(), Some(LifecycleEnd::Discarded));
        assert_eq!(stack.lifecycle.children.len(), slots);
        assert!(stack.lifecycle.children.iter().all(Option::is_none));

        open_nested(&mut stack, 2);
        stack.close_start(2, SettleStatus::Ok, None, 0).unwrap();
        stack.index = Some(0);
        stack.close_start(1, SettleStatus::Ok, None, 0).unwrap();
        stack.clear();
        stack.close_start(0, SettleStatus::Ok, None, 0).unwrap();
        assert_eq!(stack.settle_lifecycle(), Ok(Some(LifecycleEnd::Settled)));
        assert_eq!(stack.lifecycle.children.len(), slots);
    }

    #[test]
    fn test_what_an_inspector_did_reaches_the_settlement() {
        let mut stack = stack(1);
        open_nested(&mut stack, 1);
        let pending = stack.pending_start().unwrap();
        pending.set_inspector_answer();
        pending.set_outcome_rebuilt();
        let running = stack.running_start().unwrap();
        running.set_gas_replaced();
        let child = stack.close_start(1, SettleStatus::Ok, None, 0).unwrap();
        assert!(child.inspector_answer && child.outcome_rebuilt && !child.gas_replaced);
        stack.clear();
        let root = stack.close_start(0, SettleStatus::Ok, None, 0).unwrap();
        assert!(!root.inspector_answer && !root.outcome_rebuilt && root.gas_replaced);
    }
}
