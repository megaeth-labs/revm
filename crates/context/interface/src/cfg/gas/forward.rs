//! What a frame start takes out of its caller's [`GasTracker`] for the child frame, beside the
//! regular gas: a share of the EIP-8037 reservoir and of each pool.
//!
//! The caller keeps the rest while the child runs, and the child's return adds what the child left
//! back to what the caller kept ([`absorb_returned_reservoir`](GasTracker::absorb_returned_reservoir),
//! [`absorb_returned_pools`](GasTracker::absorb_returned_pools)). The shares are the schedule's
//! ([`GasParams::reservoir_forward_share`], [`GasParams::pool_forward_share`]); every schedule
//! defined here forwards the whole reservoir and every whole pool, which is upstream's forward.

use super::{GasTracker, PoolAmounts, PoolId};
use crate::cfg::GasParams;

/// What a child frame's tracker starts with beside its regular gas, taken out of its caller's: its
/// reservoir and its pools.
///
/// Returned by [`GasTracker::forward_to_child`] and [`GasTracker::forward_all_to_child`]; the
/// frame's input carries it (`reservoir` and `pool_shares`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ChildShares {
    /// The child's reservoir.
    pub reservoir: u64,
    /// The child's pools.
    pub pools: PoolAmounts,
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

    /// Takes the schedule's shares of the reservoir and of each pool out of this, the caller's,
    /// tracker for a child frame, and returns them: what the child's tracker starts with.
    ///
    /// The `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`, `CREATE` and `CREATE2` instructions
    /// call it once the frame-start hook has answered, so the shares are of the reservoir and the
    /// pools as the hook left them. A consumer that builds a child frame's input itself calls it
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
    /// the schedule's share, as `forward_to_child` does. The pools are forwarded by the schedule's
    /// shares either way.
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
        let mut pools = PoolAmounts::ZERO;
        for pool in PoolId::ALL {
            let share = params.pool_forward_share(pool, self.pool_remaining(pool));
            self.forward_pool(pool, share);
            pools.set(pool, share);
        }
        ChildShares { reservoir, pools }
    }

    /// Takes the whole reservoir and every whole pool out of this tracker for a child frame, and
    /// returns them.
    ///
    /// For the transaction's frame, which takes everything the transaction's tracker holds: the
    /// schedule's shares apply to the frames below it.
    #[inline]
    pub const fn forward_all_to_child(&mut self) -> ChildShares {
        let shares = ChildShares {
            reservoir: self.reservoir,
            pools: self.pools_remaining(),
        };
        self.reservoir = 0;
        self.set_pools_remaining(PoolAmounts::ZERO);
        shares
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildShares, GasTracker, PoolAmounts, PoolId};
    use crate::cfg::{GasId, GasParams};
    use primitives::hardfork::SpecId;

    /// A caller with a reservoir of 1,000 and pools of 1,000, 500 and 100.
    fn caller() -> GasTracker {
        let mut tracker = GasTracker::new(10_000, 10_000, 1_000);
        for (pool, amount) in PoolId::ALL.into_iter().zip([1_000, 500, 100]) {
            tracker.set_pool_remaining(pool, amount);
        }
        tracker
    }

    /// Every schedule forwards everything: the child gets the whole reservoir and every whole
    /// pool, and the caller keeps nothing of them while the child runs.
    #[test]
    fn test_every_schedule_forwards_everything() {
        for spec in [SpecId::FRONTIER, SpecId::OSAKA, SpecId::AMSTERDAM] {
            let params = GasParams::new_spec(spec);
            let mut tracker = caller();

            let shares = tracker.forward_to_child(&params);

            assert_eq!(shares.reservoir, 1_000, "{spec:?}");
            assert_eq!(
                PoolId::ALL.map(|pool| shares.pools.get(pool)),
                [1_000, 500, 100],
                "{spec:?}"
            );
            assert_eq!(tracker.reservoir(), 0, "{spec:?}");
            assert_eq!(tracker.pools_remaining(), PoolAmounts::ZERO, "{spec:?}");
            assert_eq!(
                tracker.remaining(),
                10_000,
                "{spec:?}: regular gas is not touched"
            );
        }
    }

    /// Each figure is forwarded by its own share, rounded down, and the caller keeps the rest.
    #[test]
    fn test_each_share_is_the_schedules() {
        let mut params = GasParams::new_spec(SpecId::AMSTERDAM);
        params.override_gas([
            (GasId::reservoir_kept_percent(), 2),
            (GasId::pool_kept_percent(PoolId::new(0)), 2),
            (GasId::pool_kept_percent(PoolId::new(1)), 50),
            (GasId::pool_kept_percent(PoolId::new(2)), 100),
        ]);
        let mut tracker = caller();
        assert!(tracker.record_state_cost(1));

        let shares = tracker.forward_to_child(&params);

        // The reservoir's 999 less the caller's 2%, rounded down (19), is 980; 98% of 1,000 is
        // 980; half of 500 is 250.
        assert_eq!(shares.reservoir, 980);
        assert_eq!(
            PoolId::ALL.map(|pool| shares.pools.get(pool)),
            [980, 250, 0]
        );
        assert_eq!(tracker.reservoir(), 19);
        assert_eq!(
            PoolId::ALL.map(|pool| tracker.pool_remaining(pool)),
            [20, 250, 100]
        );
        assert_eq!(tracker.state_gas_spent(), 1, "a forward is not a charge");
    }

    /// The transaction's frame takes everything, whatever the schedule.
    #[test]
    fn test_the_first_frame_takes_everything() {
        let mut tracker = caller();

        let shares = tracker.forward_all_to_child();

        assert_eq!(
            shares,
            ChildShares {
                reservoir: 1_000,
                pools: PoolAmounts::ZERO
                    .with(PoolId::new(0), 1_000)
                    .with(PoolId::new(1), 500)
                    .with(PoolId::new(2), 100),
            }
        );
        assert_eq!(tracker.reservoir(), 0);
        assert_eq!(tracker.pools_remaining(), PoolAmounts::ZERO);
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
