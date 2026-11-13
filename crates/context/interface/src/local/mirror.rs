//! A consumer-side check of the start lifecycle: a stack driven only by the lifecycle hooks, as a
//! consumer's own ledger would be, which asserts that every event the hooks receive fits it.
//!
//! revm's own context keeps one in its local context in debug builds and feeds it from its
//! [`Host`](crate::Host) hooks ([`LocalContextTr::lifecycle_mirror`](super::LocalContextTr::lifecycle_mirror));
//! a release build keeps none.

use crate::host::{FrameSettle, FrameSplit, LifecycleEnd};
use std::vec::Vec;

/// An event of the start lifecycle, as a hook received it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecycleEvent {
    /// [`Host::frame_split`](crate::Host::frame_split).
    Split(FrameSplit),
    /// [`Host::frame_settle`](crate::Host::frame_settle).
    Settle(FrameSettle),
    /// [`Host::frame_lifecycle_end`](crate::Host::frame_lifecycle_end).
    End(LifecycleEnd),
}

/// The starts the hooks reported open, innermost last, and the events of the lifecycle under way
/// or, once it ended, of the last one.
///
/// It lives in the context, so a context cloned for a simulation carries a copy. The simulation's
/// first event is its own transaction's start, which resets the copy: each execution is checked
/// from its own first start, and the original context's mirror is untouched.
#[derive(Clone, Debug, Default)]
pub struct LifecycleMirror {
    open: Vec<(u32, usize)>,
    events: Vec<LifecycleEvent>,
}

impl LifecycleMirror {
    /// Checks a split and pushes its start. The transaction's start, the one with no parent,
    /// begins a lifecycle: it must be at depth 0, and it resets the mirror. A child start's
    /// parent must be the start on top, one frame shallower.
    ///
    /// # Panics
    ///
    /// When the split does not fit.
    pub fn split(&mut self, split: &FrameSplit) {
        match split.parent {
            None => {
                assert_eq!(
                    split.depth, 0,
                    "start lifecycle: the transaction's start {} at depth {}",
                    split.id, split.depth
                );
                self.open.clear();
                self.events.clear();
            }
            Some(parent) => {
                let top = self.open.last().copied();
                assert_eq!(
                    top.map(|(id, _)| id),
                    Some(parent),
                    "start lifecycle: start {} opened under {parent} while {top:?} was on top",
                    split.id
                );
                assert_eq!(
                    top.map(|(_, depth)| depth + 1),
                    Some(split.depth),
                    "start lifecycle: start {} at depth {} under a parent at {top:?}",
                    split.id,
                    split.depth
                );
            }
        }
        self.open.push((split.id, split.depth));
        self.events.push(LifecycleEvent::Split(*split));
    }

    /// Checks a settlement and pops its start, which must be the one on top, at the same depth.
    ///
    /// # Panics
    ///
    /// When the settlement does not fit.
    pub fn settle(&mut self, settle: &FrameSettle) {
        let top = self.open.pop();
        assert_eq!(
            top,
            Some((settle.id, settle.depth)),
            "start lifecycle: settled ({}, depth {}) while {top:?} was on top",
            settle.id,
            settle.depth
        );
        self.events.push(LifecycleEvent::Settle(*settle));
    }

    /// Checks the end of a lifecycle: a settled one leaves no start open. Either end empties the
    /// stack.
    ///
    /// # Panics
    ///
    /// When a lifecycle ends settled with a start open.
    pub fn end(&mut self, end: LifecycleEnd) {
        if end == LifecycleEnd::Settled {
            assert!(
                self.open.is_empty(),
                "start lifecycle: settled with starts open: {:?}",
                self.open
            );
        }
        self.open.clear();
        self.events.push(LifecycleEvent::End(end));
    }

    /// The events of the lifecycle under way or, once it ended, of the last one.
    pub fn events(&self) -> &[LifecycleEvent] {
        &self.events
    }

    /// The starts open, innermost last, as `(identity, depth)`.
    pub fn open(&self) -> &[(u32, usize)] {
        &self.open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::SettleStatus;

    fn split(id: u32, parent: Option<u32>, depth: usize) -> FrameSplit {
        FrameSplit { id, parent, depth }
    }

    fn settle(id: u32, depth: usize) -> FrameSettle {
        FrameSettle {
            id,
            depth,
            status: SettleStatus::Ok,
            created: None,
            inspector_answer: false,
            gas_replaced: false,
            outcome_rebuilt: false,
            held_reservoir: 0,
        }
    }

    #[test]
    fn test_a_paired_lifecycle_fits() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.split(&split(2, Some(1), 1));
        mirror.settle(&settle(2, 1));
        mirror.settle(&settle(1, 0));
        mirror.end(LifecycleEnd::Settled);
        assert!(mirror.open().is_empty());
        assert_eq!(mirror.events().len(), 5);
    }

    #[test]
    fn test_the_transactions_start_resets_what_a_clone_brought() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.split(&split(2, Some(1), 1));
        let mut simulation = mirror.clone();
        simulation.split(&split(1, None, 0));
        assert_eq!(simulation.open(), &[(1, 0)]);
        assert_eq!(
            simulation.events(),
            &[LifecycleEvent::Split(split(1, None, 0))]
        );
        assert_eq!(
            mirror.open(),
            &[(1, 0), (2, 1)],
            "the original is untouched"
        );
    }

    #[test]
    fn test_a_discarded_lifecycle_may_leave_starts_open() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.end(LifecycleEnd::Discarded);
        assert!(mirror.open().is_empty());
    }

    #[test]
    #[should_panic(expected = "the transaction's start 1 at depth 5")]
    fn test_a_transaction_start_off_depth_zero_does_not_fit() {
        LifecycleMirror::default().split(&split(1, None, 5));
    }

    #[test]
    #[should_panic(expected = "start 2 at depth 3 under a parent at Some((1, 0))")]
    fn test_a_child_off_its_parents_depth_does_not_fit() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.split(&split(2, Some(1), 3));
    }

    #[test]
    #[should_panic(expected = "start 3 opened under 1 while Some((2, 1)) was on top")]
    fn test_a_child_under_another_start_does_not_fit() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.split(&split(2, Some(1), 1));
        mirror.split(&split(3, Some(1), 1));
    }

    #[test]
    #[should_panic(expected = "settled (1, depth 0) while Some((2, 1)) was on top")]
    fn test_a_settlement_out_of_order_does_not_fit() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.split(&split(2, Some(1), 1));
        mirror.settle(&settle(1, 0));
    }

    #[test]
    #[should_panic(expected = "settled (1, depth 27) while Some((1, 0)) was on top")]
    fn test_a_settlement_off_its_splits_depth_does_not_fit() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.settle(&settle(1, 27));
    }

    #[test]
    #[should_panic(expected = "settled with starts open: [(1, 0)]")]
    fn test_a_settled_end_with_a_start_open_does_not_fit() {
        let mut mirror = LifecycleMirror::default();
        mirror.split(&split(1, None, 0));
        mirror.end(LifecycleEnd::Settled);
    }
}
