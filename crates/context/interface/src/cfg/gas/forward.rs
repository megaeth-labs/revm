//! What a frame start takes out of its caller's [`GasTracker`] for the child frame, beside the
//! regular gas: a share of the EIP-8037 reservoir.
//!
//! The caller keeps the rest while the child runs, and the child's return adds what the child left
//! back to what the caller kept ([`absorb_returned_reservoir`](GasTracker::absorb_returned_reservoir)).
//! The share is the schedule's ([`GasParams::reservoir_forward_share`]); every schedule defined
//! here forwards the whole reservoir, which is upstream's forward.

use super::GasTracker;
use crate::cfg::GasParams;

/// What a child frame's tracker starts with beside its regular gas, taken out of its caller's: its
/// reservoir.
///
/// Returned by [`GasTracker::forward_to_child`] and [`GasTracker::forward_all_to_child`]; the
/// frame's input carries it (`reservoir`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ChildShares {
    /// The child's reservoir.
    pub reservoir: u64,
}

impl GasTracker {
    /// Takes `share` out of the reservoir for a child frame, which starts with it as its
    /// reservoir.
    ///
    /// Not a charge: no counter moves, since the share comes back when the child returns
    /// ([`absorb_returned_reservoir`](Self::absorb_returned_reservoir)). `share` is at most the
    /// reservoir, which the schedule's forward share guarantees; more saturates at an empty
    /// reservoir.
    #[inline]
    pub const fn forward_reservoir(&mut self, share: u64) {
        self.reservoir = self.reservoir.saturating_sub(share);
    }

    /// Takes the schedule's share of the reservoir out of this, the caller's, tracker for a child
    /// frame, and returns it: what the child's tracker starts with.
    ///
    /// The `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`, `CREATE` and `CREATE2` instructions
    /// call it once the frame-start hook has answered, so the share is of the reservoir as the
    /// hook left it. A consumer that builds a child frame's input itself calls it
    /// too, so that the child's return, which adds what the child left to what the caller kept,
    /// gives the caller back exactly what it forwarded.
    #[inline]
    pub fn forward_to_child(&mut self, params: &GasParams) -> ChildShares {
        self.forward_to_child_with_reservoir(params, None)
    }

    /// [`forward_to_child`](Self::forward_to_child), for a start whose child reservoir the
    /// frame-start hook may have named (`FrameStart::reservoir`).
    ///
    /// `Some(reservoir)` is the child's reservoir, which the hook took out of this tracker itself:
    /// no share of the reservoir is taken here, and the child starts with `reservoir`. `None` takes
    /// the schedule's share, as `forward_to_child` does.
    #[inline]
    pub fn forward_to_child_with_reservoir(
        &mut self,
        params: &GasParams,
        reservoir: Option<u64>,
    ) -> ChildShares {
        let reservoir = reservoir.unwrap_or_else(|| {
            let share = params.reservoir_forward_share(self.reservoir);
            self.forward_reservoir(share);
            share
        });
        ChildShares { reservoir }
    }

    /// Takes the whole reservoir out of this tracker for a child frame, and returns it.
    ///
    /// For the transaction's frame, which takes the transaction's whole reservoir: the schedule's
    /// share applies to the frames below it.
    #[inline]
    pub const fn forward_all_to_child(&mut self) -> ChildShares {
        let shares = ChildShares {
            reservoir: self.reservoir,
        };
        self.reservoir = 0;
        shares
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildShares, GasTracker};
    use crate::cfg::{GasId, GasParams};
    use primitives::hardfork::SpecId;

    /// A caller with a reservoir of 1,000.
    fn caller() -> GasTracker {
        GasTracker::new(10_000, 10_000, 1_000)
    }

    /// Every schedule forwards the whole reservoir, and the caller keeps none of it while the
    /// child runs.
    #[test]
    fn test_every_schedule_forwards_everything() {
        for spec in [SpecId::FRONTIER, SpecId::OSAKA, SpecId::AMSTERDAM] {
            let params = GasParams::new_spec(spec);
            let mut tracker = caller();

            let shares = tracker.forward_to_child(&params);

            assert_eq!(shares.reservoir, 1_000, "{spec:?}");
            assert_eq!(tracker.reservoir(), 0, "{spec:?}");
            assert_eq!(
                tracker.remaining(),
                10_000,
                "{spec:?}: regular gas is not touched"
            );
        }
    }

    /// The reservoir is forwarded by the schedule's share, and the caller keeps the rest.
    #[test]
    fn test_each_share_is_the_schedules() {
        let mut params = GasParams::new_spec(SpecId::AMSTERDAM);
        params.override_gas([(GasId::reservoir_kept_percent(), 2)]);
        let mut tracker = caller();
        assert!(tracker.record_state_cost(1));

        let shares = tracker.forward_to_child(&params);

        // The reservoir's 999 less the caller's 2%, rounded down (19), is 980.
        assert_eq!(shares.reservoir, 980);
        assert_eq!(tracker.reservoir(), 19);
        assert_eq!(tracker.state_gas_spent(), 1, "a forward is not a charge");
    }

    /// The transaction's frame takes the whole reservoir, whatever the schedule.
    #[test]
    fn test_the_first_frame_takes_everything() {
        let mut tracker = caller();

        let shares = tracker.forward_all_to_child();

        assert_eq!(shares, ChildShares { reservoir: 1_000 });
        assert_eq!(tracker.reservoir(), 0);
    }

    /// A caller that kept part of its reservoir when it forwarded a share gets the child's
    /// reservoir added to what it kept, and the returned reservoir, not the kept part, unwinds
    /// the caller's outstanding spill. With nothing kept the result is the child's reservoir, as
    /// a settlement that replaced the caller's reservoir gave.
    #[test]
    fn test_a_returned_reservoir_joins_the_part_the_caller_kept() {
        // Kept 20 of 1,000, the child returns 700 of its 980.
        let mut kept = GasTracker::new(1_000, 1_000, 1_000);
        kept.forward_reservoir(980);
        kept.absorb_returned_reservoir(700);
        assert_eq!((kept.reservoir(), kept.remaining()), (720, 1_000));

        // A spill of 150 outstanding (a successful child's, merged before the absorb): the
        // returned 100 unwinds 100 of it, and the 20 kept stays in the reservoir.
        let mut spilled = GasTracker::new(1_000, 850, 20);
        spilled.add_state_gas_spilled(150);
        spilled.absorb_returned_reservoir(100);
        assert_eq!(spilled.reservoir(), 20);
        assert_eq!(spilled.remaining(), 950);
        assert_eq!(spilled.state_gas_spilled(), 50);

        // Nothing kept: the child's reservoir, whatever the caller held before the forward.
        let mut whole = GasTracker::new(1_000, 1_000, 1_000);
        whole.forward_reservoir(1_000);
        whole.absorb_returned_reservoir(300);
        assert_eq!(whole.reservoir(), 300);
    }

    /// A forward above the reservoir saturates at an empty reservoir.
    #[test]
    fn test_a_reservoir_forward_saturates() {
        let mut tracker = caller();
        tracker.forward_reservoir(5_000);
        assert_eq!(tracker.reservoir(), 0);
    }
}
