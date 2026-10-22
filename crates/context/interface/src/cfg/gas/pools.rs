//! The resource pools of a [`GasTracker`]: budgets beside the gas, each forwarded to a child frame
//! by its own share and settled with the frame's gas.
//!
//! A tracker carries [`PoolId::COUNT`] pools, which a consumer gives their meaning (bytes appended,
//! records written, entries created); the fork knows them only by index. Each pool is a figure
//! beside the gas: `pool_remaining` is what the frame may still draw, and `pool_spent` the frame's
//! net draw so far, signed as the state and history nets are. A draw
//! ([`record_pool`](GasTracker::record_pool)) the pool cannot pay fails, takes nothing and sets
//! the pool's crossing ([`pool_crossing`](GasTracker::pool_crossing)). A frame that reverts or
//! halts gives its draws on every pool back ([`rollback_state_gas`](GasTracker::rollback_state_gas)),
//! and a frame that succeeds hands its nets to its caller, so a caller's rollback undoes its
//! children's draws too. Every pool starts unlimited (`u64::MAX`), and nothing in this workspace
//! draws one, so a consumer that sets no pool sees nothing of them.
//!
//! The nets are `i64`, as the gas nets are, while draws and refills take `u64` amounts. As the
//! tracker does for gas amounts, it assumes that every draw and refill is at most `i64::MAX`, and
//! that each net, with the nets of the children merged into it, stays within `±i64::MAX`, so it can
//! be negated ([`adopt_pool_share`](GasTracker::adopt_pool_share)); a pool itself may be anything
//! up to the unlimited `u64::MAX`. A release build does not check it: past it the net saturates,
//! and a rollback no longer restores the pool exactly. A debug build asserts it wherever a net
//! moves ([`record_pool`](GasTracker::record_pool), [`refill_pool`](GasTracker::refill_pool),
//! [`add_pool_spent`](GasTracker::add_pool_spent)). Reaching it takes about 9.2 × 10^18 units.

use super::{saturating_signed, GasTracker};

/// One of a tracker's resource pools, by index.
///
/// A consumer names the pools it uses with constants (`const PAGES: PoolId = PoolId::new(0);`);
/// [`new`](Self::new) refuses an index the tracker has no pool for, so such a constant does not
/// compile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PoolId(u8);

impl PoolId {
    /// How many pools a tracker carries.
    pub const COUNT: usize = 3;

    /// Every pool, in index order.
    pub const ALL: [Self; Self::COUNT] = [Self(0), Self(1), Self(2)];

    /// The pool at `index`.
    ///
    /// # Panics
    ///
    /// If `index` is not below [`COUNT`](Self::COUNT); in a constant, that is a compile error.
    #[inline]
    pub const fn new(index: usize) -> Self {
        assert!(index < Self::COUNT, "a tracker has no pool at this index");
        Self(index as u8)
    }

    /// The pool's index, below [`COUNT`](Self::COUNT).
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// An amount for each of a tracker's pools: the shares a child frame's pools start with
/// ([`ChildShares::pools`](super::ChildShares::pools)), or what a frame start drew from its
/// caller's pools ([`StartCharge::pools`](super::StartCharge::pools)).
///
/// Encoded as a sequence of one number per pool, in index order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct PoolAmounts([u64; PoolId::COUNT]);

impl PoolAmounts {
    /// Nothing in any pool. The default.
    pub const ZERO: Self = Self([0; PoolId::COUNT]);

    /// Every pool unlimited: what a tracker starts with, and the share of a child whose caller
    /// limits nothing.
    pub const UNLIMITED: Self = Self([u64::MAX; PoolId::COUNT]);

    /// The amount for `pool`.
    #[inline]
    pub const fn get(&self, pool: PoolId) -> u64 {
        self.0[pool.index()]
    }

    /// Sets the amount for `pool`.
    #[inline]
    pub const fn set(&mut self, pool: PoolId, amount: u64) {
        self.0[pool.index()] = amount;
    }

    /// These amounts, with `amount` for `pool`.
    #[inline]
    #[must_use]
    pub const fn with(mut self, pool: PoolId, amount: u64) -> Self {
        self.set(pool, amount);
        self
    }

    /// Whether every amount is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        let mut index = 0;
        while index < PoolId::COUNT {
            if self.0[index] != 0 {
                return false;
            }
            index += 1;
        }
        true
    }
}

impl GasTracker {
    /// The pool: what this frame may still draw from it.
    ///
    /// Starts at `u64::MAX`, an unlimited pool, in every tracker. A consumer that limits the pool
    /// sets the transaction's ([`set_pool_remaining`](Self::set_pool_remaining)); every child
    /// frame then receives a share of its caller's pool when it starts, and the caller keeps the
    /// rest.
    #[inline]
    pub const fn pool_remaining(&self, pool: PoolId) -> u64 {
        self.pools_remaining[pool.index()]
    }

    /// Sets the pool. Its net draw ([`pool_spent`](Self::pool_spent)) is left as it is.
    #[inline]
    pub const fn set_pool_remaining(&mut self, pool: PoolId, val: u64) {
        self.pools_remaining[pool.index()] = val;
    }

    /// Every pool: what this frame may still draw from each.
    #[inline]
    pub const fn pools_remaining(&self) -> PoolAmounts {
        PoolAmounts(self.pools_remaining)
    }

    /// Sets every pool, as a child frame's are set to the shares its caller forwarded. The nets
    /// are left as they are.
    #[inline]
    pub const fn set_pools_remaining(&mut self, pools: PoolAmounts) {
        self.pools_remaining = pools.0;
    }

    /// The frame's net draw of the pool so far: draws less refills, plus the nets of the children
    /// that succeeded.
    ///
    /// Negative while the frame has refilled more than it drew, as
    /// [`history_gas_spent`](Self::history_gas_spent) can be. A rollback adds it back to the pool.
    /// It is exact while it stays within `±i64::MAX` and no single draw or refill exceeds
    /// `i64::MAX`, which a debug build asserts.
    #[inline]
    pub const fn pool_spent(&self, pool: PoolId) -> i64 {
        self.pools_spent[pool.index()]
    }

    /// Sets the pool's net draw.
    #[inline]
    pub const fn set_pool_spent(&mut self, pool: PoolId, val: i64) {
        self.pools_spent[pool.index()] = val;
    }

    /// Adds `delta` to the pool's net draw, saturating. Used to merge a successful child's net
    /// into its caller, so a later rollback of the caller undoes the child's draws too.
    #[inline]
    pub const fn add_pool_spent(&mut self, pool: PoolId, delta: i64) {
        let spent = &mut self.pools_spent[pool.index()];
        debug_assert!(
            matches!(spent.checked_add(delta), Some(net) if net != i64::MIN),
            "a pool's net leaves the signed range"
        );
        *spent = spent.saturating_add(delta);
    }

    /// Draws `cost` from the pool.
    ///
    /// Returns `false` if the pool cannot pay, leaving it untouched and setting its crossing
    /// ([`pool_crossing`](Self::pool_crossing)). A draw of zero always succeeds. The draw is
    /// counted on [`pool_spent`](Self::pool_spent), so a rollback gives it back.
    #[inline]
    #[must_use = "a draw the pool cannot pay is the consumer's stop"]
    pub const fn record_pool(&mut self, pool: PoolId, cost: u64) -> bool {
        let index = pool.index();
        if let Some(left) = self.pools_remaining[index].checked_sub(cost) {
            debug_assert!(
                cost <= i64::MAX as u64
                    && self.pools_spent[index].checked_add(cost as i64).is_some(),
                "a pool's net leaves the signed range"
            );
            self.pools_remaining[index] = left;
            self.pools_spent[index] =
                self.pools_spent[index].saturating_add(saturating_signed(cost));
            return true;
        }
        self.pool_crossings |= 1 << index;
        false
    }

    /// Gives `amount` back to the pool, for a draw the frame took back.
    ///
    /// The net draw falls by the same amount, below zero when the matching draw was a caller's,
    /// which the merge on return reconciles, as [`refill_history`](Self::refill_history) does.
    #[inline]
    pub const fn refill_pool(&mut self, pool: PoolId, amount: u64) {
        let index = pool.index();
        debug_assert!(
            amount <= i64::MAX as u64
                && matches!(
                    self.pools_spent[index].checked_sub(amount as i64),
                    Some(net) if net != i64::MIN
                ),
            "a pool's net leaves the signed range"
        );
        self.pools_remaining[index] = self.pools_remaining[index].saturating_add(amount);
        self.pools_spent[index] = self.pools_spent[index].saturating_sub(saturating_signed(amount));
    }

    /// Takes `share` out of the pool for a child frame, which starts with it as its pool.
    ///
    /// Not a draw: the net is unchanged, since the share comes back when the child returns
    /// ([`absorb_returned_pool`](Self::absorb_returned_pool)). `share` is at most the pool, which
    /// the schedule's forward share guarantees; more saturates at an empty pool.
    #[inline]
    pub const fn forward_pool(&mut self, pool: PoolId, share: u64) {
        let remaining = &mut self.pools_remaining[pool.index()];
        *remaining = remaining.saturating_sub(share);
    }

    /// Adds back what a returning child left of its pool: its remaining pool when it succeeded,
    /// the whole share it was given when it reverted or halted, since its rollback restored it.
    #[inline]
    pub const fn absorb_returned_pool(&mut self, pool: PoolId, remaining: u64) {
        let own = &mut self.pools_remaining[pool.index()];
        *own = own.saturating_add(remaining);
    }

    /// Takes `share` as the pool this tracker started with: the pool becomes `share` less the net
    /// drawn so far, so that the pool and the net add up to `share`, as a frame's do.
    ///
    /// For a tracker built outside a frame, such as a precompile's answer, which may have drawn
    /// before it learns the share its caller forwarded. A rollback then restores `share`. A net
    /// above `share` leaves an empty pool.
    #[inline]
    pub const fn adopt_pool_share(&mut self, pool: PoolId, share: u64) {
        let index = pool.index();
        self.pools_remaining[index] =
            share.saturating_add_signed(self.pools_spent[index].saturating_neg());
    }

    /// [`adopt_pool_share`](Self::adopt_pool_share) for every pool.
    #[inline]
    pub const fn adopt_pool_shares(&mut self, shares: PoolAmounts) {
        let mut index = 0;
        while index < PoolId::COUNT {
            self.adopt_pool_share(PoolId::ALL[index], shares.0[index]);
            index += 1;
        }
    }

    /// Settles every pool of a returning child into this, its caller's, tracker: the child's nets
    /// merge into this tracker's when the child `succeeded`, and what the child left of each pool
    /// is absorbed back whatever its end, as [`absorb_returned_pool`](Self::absorb_returned_pool)
    /// says. A child that failed has already rolled its draws back.
    #[inline]
    pub const fn absorb_returned_pools(&mut self, child: &Self, succeeded: bool) {
        let mut index = 0;
        while index < PoolId::COUNT {
            let pool = PoolId::ALL[index];
            if succeeded {
                self.add_pool_spent(pool, child.pools_spent[index]);
            }
            self.absorb_returned_pool(pool, child.pools_remaining[index]);
            index += 1;
        }
    }

    /// The pool's crossing: whether a draw failed that the pool could not pay, since the record
    /// was last cleared, or a consumer set it with [`set_pool_crossing`](Self::set_pool_crossing).
    ///
    /// A consumer reads it from a halted frame's result to tell a frame that ran out of the pool
    /// from one that ran out of gas, and turns it into a stop of its own class. The record says
    /// that a draw failed, not what it asked for: the hook that drew knows its cost when the draw
    /// fails. The failed draw took nothing, and no halt touches a pool, so the pool itself still
    /// reads as it was at the failure until the frame's rollback. The record survives
    /// [`spend_all`](Self::spend_all), starts empty in every child, and is not taken over by a
    /// caller. Each pool's is a bit beside the failed-charge record
    /// ([`failed_charge_remaining`](Self::failed_charge_remaining)), and each is set and cleared
    /// apart from the others.
    #[inline]
    pub const fn pool_crossing(&self, pool: PoolId) -> bool {
        self.pool_crossings & (1 << pool.index()) != 0
    }

    /// Clears the record returned by [`pool_crossing`](Self::pool_crossing).
    #[inline]
    pub const fn clear_pool_crossing(&mut self, pool: PoolId) {
        self.pool_crossings &= !(1 << pool.index());
    }

    /// Sets the record returned by [`pool_crossing`](Self::pool_crossing); `false` clears it. The
    /// pool is left as it is.
    ///
    /// For a consumer that marks a result it produced itself, such as an answer it gave without
    /// running a frame, as a failed draw, so a classifier reading the record treats it as one the
    /// tracker recorded.
    #[inline]
    pub const fn set_pool_crossing(&mut self, pool: PoolId, crossing: bool) {
        if crossing {
            self.pool_crossings |= 1 << pool.index();
        } else {
            self.clear_pool_crossing(pool);
        }
    }

    /// Gives every pool's net draw back to it, which then reads as the pool the frame started
    /// with, and zeroes the nets. Part of [`rollback_state_gas`](Self::rollback_state_gas).
    #[inline]
    pub(super) const fn rollback_pools(&mut self) {
        let mut index = 0;
        while index < PoolId::COUNT {
            self.pools_remaining[index] =
                self.pools_remaining[index].saturating_add_signed(self.pools_spent[index]);
            self.pools_spent[index] = 0;
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GasTracker, PoolAmounts, PoolId};

    /// A tracker with every pool at 1,000.
    fn pools_1000() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 1_000, 300);
        for pool in PoolId::ALL {
            tracker.set_pool_remaining(pool, 1_000);
        }
        tracker
    }

    /// The pools other than `pool` as they were set: 1,000, nothing drawn, no record.
    fn assert_untouched_but(tracker: &GasTracker, pool: PoolId) {
        for other in PoolId::ALL.into_iter().filter(|&other| other != pool) {
            assert_eq!(tracker.pool_remaining(other), 1_000, "{other:?}");
            assert_eq!(tracker.pool_spent(other), 0, "{other:?}");
            assert!(!tracker.pool_crossing(other), "{other:?}");
        }
    }

    /// The tracker has the pools `ALL` names, each by its index; an index past them is refused.
    #[test]
    fn test_the_pools_are_named_by_index() {
        assert_eq!(PoolId::COUNT, 3);
        for (index, pool) in PoolId::ALL.into_iter().enumerate() {
            assert_eq!(pool, PoolId::new(index));
            assert_eq!(pool.index(), index);
        }
    }

    #[test]
    #[should_panic(expected = "a tracker has no pool at this index")]
    fn test_an_index_past_the_pools_is_refused() {
        let _ = PoolId::new(PoolId::COUNT);
    }

    /// Amounts are read and set by pool; zero is the default and unlimited is every pool at
    /// `u64::MAX`.
    #[test]
    fn test_pool_amounts_are_read_and_set_by_pool() {
        assert_eq!(PoolAmounts::default(), PoolAmounts::ZERO);
        assert!(PoolAmounts::ZERO.is_zero());
        let amounts = PoolAmounts::ZERO.with(PoolId::new(1), 40);
        assert!(!amounts.is_zero());
        assert_eq!(
            PoolId::ALL.map(|pool| amounts.get(pool)),
            [0, 40, 0],
            "only the pool set moves"
        );
        assert_eq!(
            PoolId::ALL.map(|pool| PoolAmounts::UNLIMITED.get(pool)),
            [u64::MAX; 3]
        );
    }

    /// Every tracker starts with every pool unlimited, nothing drawn and no record; `Default` is
    /// `new(0, 0, 0)`.
    #[test]
    fn test_a_new_tracker_has_unlimited_pools() {
        let tracker = GasTracker::new(1_000, 1_000, 300);
        assert_eq!(tracker.pools_remaining(), PoolAmounts::UNLIMITED);
        for pool in PoolId::ALL {
            assert_eq!(tracker.pool_spent(pool), 0);
            assert!(!tracker.pool_crossing(pool));
        }
        assert_eq!(GasTracker::default(), GasTracker::new(0, 0, 0));
        assert_eq!(
            GasTracker::default().pools_remaining(),
            PoolAmounts::UNLIMITED
        );
    }

    /// Each pool is a figure of its own: a draw, a failed draw, a refill, a forward and a record
    /// on one leave the others as they were.
    #[test]
    fn test_each_pool_moves_alone() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(tracker.record_pool(pool, 300));
            assert!(!tracker.record_pool(pool, 800));
            tracker.refill_pool(pool, 50);
            tracker.forward_pool(pool, 100);
            tracker.absorb_returned_pool(pool, 20);
            tracker.add_pool_spent(pool, 7);

            assert_eq!(tracker.pool_remaining(pool), 670, "{pool:?}");
            assert_eq!(tracker.pool_spent(pool), 257, "{pool:?}");
            assert!(tracker.pool_crossing(pool), "{pool:?}");
            assert_untouched_but(&tracker, pool);

            tracker.clear_pool_crossing(pool);
            assert!(!tracker.pool_crossing(pool), "{pool:?}");
            tracker.set_pool_crossing(pool, true);
            assert_untouched_but(&tracker, pool);
        }
    }

    /// A pool's crossing shares a word with the failed-charge record and is a record of its own: a
    /// failed draw records no failed charge, a failed charge records no crossing, and setting or
    /// clearing either leaves the other as it is.
    #[test]
    fn test_the_crossings_and_the_failed_charge_record_are_independent() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(!tracker.record_pool(pool, 1_001));
            assert!(tracker.pool_crossing(pool));
            assert_eq!(tracker.failed_charge_remaining(), None);

            tracker.clear_pool_crossing(pool);
            assert!(!tracker.record_regular_cost(1_001));
            assert_eq!(tracker.failed_charge_remaining(), Some(1_000));
            assert!(!tracker.pool_crossing(pool));

            tracker.set_pool_crossing(pool, true);
            tracker.set_failed_charge_remaining(None);
            assert!(tracker.pool_crossing(pool));
            tracker.set_failed_charge_remaining(Some(0));
            tracker.clear_pool_crossing(pool);
            assert_eq!(tracker.failed_charge_remaining(), Some(0));
        }
    }

    #[test]
    fn test_record_pool_draws_the_pool_and_counts_the_draw() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();

            assert!(tracker.record_pool(pool, 300));
            assert!(tracker.record_pool(pool, 200));

            assert_eq!(tracker.pool_remaining(pool), 500);
            assert_eq!(tracker.pool_spent(pool), 500);
            assert!(!tracker.pool_crossing(pool));
        }
    }

    /// A draw the pool cannot pay takes nothing and leaves the record.
    #[test]
    fn test_a_draw_beyond_the_pool_fails_and_records_the_crossing() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(tracker.record_pool(pool, 900));

            assert!(!tracker.record_pool(pool, 101));

            assert_eq!(
                tracker.pool_remaining(pool),
                100,
                "the failed draw took nothing"
            );
            assert_eq!(tracker.pool_spent(pool), 900);
            assert!(tracker.pool_crossing(pool));
        }
    }

    /// A draw of exactly the pool succeeds, and a draw of zero succeeds on an empty pool.
    #[test]
    fn test_a_draw_of_the_whole_pool_succeeds_and_zero_always_does() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(tracker.record_pool(pool, 1_000));
            assert_eq!(tracker.pool_remaining(pool), 0);
            assert!(tracker.record_pool(pool, 0));
            assert!(!tracker.pool_crossing(pool));
            assert!(!tracker.record_pool(pool, 1));
            assert!(tracker.pool_crossing(pool));
        }
    }

    /// The record stays until it is cleared: a later draw that succeeds leaves it, and a consumer
    /// clears or sets it, which leaves the pool as it is.
    #[test]
    fn test_the_record_stays_until_cleared() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(!tracker.record_pool(pool, 1_001));
            assert!(tracker.record_pool(pool, 10));
            assert!(tracker.pool_crossing(pool));

            tracker.clear_pool_crossing(pool);
            assert!(!tracker.pool_crossing(pool));

            tracker.set_pool_crossing(pool, true);
            assert!(tracker.pool_crossing(pool));
            tracker.set_pool_crossing(pool, false);
            assert!(!tracker.pool_crossing(pool));
            assert_eq!(
                tracker.pool_remaining(pool),
                990,
                "setting the record leaves the pool"
            );
        }
    }

    /// The halt that follows a failed draw zeroes the regular gas and nothing else: the record
    /// and the pool are still readable after it.
    #[test]
    fn test_the_record_and_the_pool_survive_spend_all() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(tracker.record_pool(pool, 600));
            assert!(!tracker.record_pool(pool, 500));

            tracker.spend_all();

            assert_eq!(tracker.remaining(), 0);
            assert_eq!(tracker.pool_remaining(pool), 400);
            assert!(tracker.pool_crossing(pool));
        }
    }

    /// A refill gives back and lowers the net, below zero when the draw was a caller's.
    #[test]
    fn test_refill_pool_gives_back_and_can_take_the_net_below_zero() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            assert!(tracker.record_pool(pool, 300));

            tracker.refill_pool(pool, 500);

            assert_eq!(tracker.pool_remaining(pool), 1_200);
            assert_eq!(tracker.pool_spent(pool), -200);
        }
    }

    /// Rolling a frame back gives every pool's net draw back, so each reads as the pool the frame
    /// started with, and nothing else of the rollback changes.
    #[test]
    fn test_rollback_restores_every_pool_the_frame_started_with() {
        let mut tracker = pools_1000();
        assert!(tracker.record_state_cost(200));
        let [first, second, third] = PoolId::ALL;
        assert!(tracker.record_pool(first, 300));
        tracker.refill_pool(first, 100);
        assert!(tracker.record_pool(first, 50));
        assert!(tracker.record_pool(second, 999));
        tracker.refill_pool(third, 40);
        assert_eq!(
            PoolId::ALL.map(|pool| tracker.pool_remaining(pool)),
            [750, 1, 1_040]
        );

        tracker.rollback_state_gas();

        for pool in PoolId::ALL {
            assert_eq!(tracker.pool_remaining(pool), 1_000, "{pool:?}");
            assert_eq!(tracker.pool_spent(pool), 0, "{pool:?}");
        }
        assert_eq!(tracker.reservoir(), 300, "the state rollback is as it was");
        assert_eq!(tracker.state_gas_spent(), 0);
    }

    /// A rollback of a frame that refilled a caller's draw takes the refill back out.
    #[test]
    fn test_rollback_undoes_a_refill_of_a_callers_draw() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            tracker.refill_pool(pool, 100);
            assert_eq!(tracker.pool_remaining(pool), 1_100);

            tracker.rollback_state_gas();

            assert_eq!(tracker.pool_remaining(pool), 1_000);
            assert_eq!(tracker.pool_spent(pool), 0);
        }
    }

    /// The forward takes the share out of the pool without counting it as a draw; the child's
    /// return adds back what it left.
    #[test]
    fn test_forward_takes_the_share_and_the_return_adds_back_what_the_child_left() {
        for pool in PoolId::ALL {
            let mut parent = pools_1000();
            parent.forward_pool(pool, 980);
            assert_eq!(parent.pool_remaining(pool), 20);
            assert_eq!(parent.pool_spent(pool), 0, "a forward is not a draw");

            parent.absorb_returned_pool(pool, 700);
            assert_eq!(parent.pool_remaining(pool), 720);
        }
    }

    /// A child's pools settle into its caller's: on success its nets merge and what it left comes
    /// back; on failure its rollback gave its draws back first, and no net merges.
    #[test]
    fn test_the_pools_of_a_returning_child_settle_into_its_caller() {
        for kept in [true, false] {
            let mut parent = pools_1000();
            let shares = [900, 500, 1_000];
            let mut child = GasTracker::new(500, 500, 0);
            for (pool, share) in PoolId::ALL.into_iter().zip(shares) {
                parent.forward_pool(pool, share);
                child.set_pool_remaining(pool, share);
            }
            let [first, second, third] = PoolId::ALL;
            assert!(child.record_pool(first, 100));
            assert!(child.record_pool(second, 20));
            child.refill_pool(third, 5);
            if !kept {
                child.rollback_state_gas();
            }

            parent.absorb_returned_pools(&child, kept);

            let (pools, spent) = if kept {
                ([900, 980, 1_005], [100, 20, -5])
            } else {
                ([1_000; 3], [0; 3])
            };
            assert_eq!(
                PoolId::ALL.map(|pool| parent.pool_remaining(pool)),
                pools,
                "kept: {kept}"
            );
            assert_eq!(
                PoolId::ALL.map(|pool| parent.pool_spent(pool)),
                spent,
                "kept: {kept}"
            );
        }
    }

    /// A tracker that adopts a share keeps its net and takes the share less the net as its pool,
    /// so a rollback restores the share: after draws, after a refill below zero, and after none.
    #[test]
    fn test_an_adopted_share_less_the_net_is_the_pool() {
        for pool in PoolId::ALL {
            for (name, draws, refills, expected) in [
                ("drew", 40, 0, 860),
                ("refilled", 0, 25, 925),
                ("nothing", 0, 0, 900),
            ] {
                let mut answer = GasTracker::new(500, 500, 0);
                assert!(answer.record_pool(pool, draws));
                answer.refill_pool(pool, refills);
                let spent = answer.pool_spent(pool);

                answer.adopt_pool_share(pool, 900);

                assert_eq!(answer.pool_remaining(pool), expected, "{pool:?} {name}");
                assert_eq!(
                    answer.pool_spent(pool),
                    spent,
                    "{pool:?} {name}: the net is kept"
                );
                answer.rollback_state_gas();
                assert_eq!(
                    answer.pool_remaining(pool),
                    900,
                    "{pool:?} {name}: a rollback restores the share"
                );
            }
        }
    }

    /// Every pool adopts its own share.
    #[test]
    fn test_adopted_shares_go_to_their_pools() {
        let mut answer = GasTracker::new(500, 500, 0);
        let [first, ..] = PoolId::ALL;
        assert!(answer.record_pool(first, 40));

        answer.adopt_pool_shares(PoolAmounts::ZERO.with(first, 900).with(PoolId::new(2), 7));

        assert_eq!(
            PoolId::ALL.map(|pool| answer.pool_remaining(pool)),
            [860, 0, 7]
        );
    }

    /// A forward above the pool and a return above `u64::MAX` saturate rather than wrap.
    #[test]
    fn test_forward_and_return_saturate() {
        for pool in PoolId::ALL {
            let mut tracker = pools_1000();
            tracker.forward_pool(pool, 5_000);
            assert_eq!(tracker.pool_remaining(pool), 0);

            let mut unlimited = GasTracker::new(1, 1, 0);
            unlimited.absorb_returned_pool(pool, 1);
            assert_eq!(unlimited.pool_remaining(pool), u64::MAX);
        }
    }

    /// The net is exact up to `i64::MAX`: a draw of it from the unlimited pool, a refill of it,
    /// and a child's net of it merged into its caller each roll back to the pool they started
    /// from.
    #[test]
    fn test_the_net_is_exact_up_to_the_signed_maximum() {
        const MAX: u64 = i64::MAX as u64;
        for pool in PoolId::ALL {
            let mut drawn = GasTracker::new(1_000, 1_000, 0);
            assert!(drawn.record_pool(pool, MAX));
            assert_eq!(drawn.pool_spent(pool), i64::MAX);
            drawn.rollback_state_gas();
            assert_eq!(drawn.pool_remaining(pool), u64::MAX);

            let mut refilled = GasTracker::new(1_000, 1_000, 0);
            refilled.set_pool_remaining(pool, 0);
            refilled.refill_pool(pool, MAX);
            assert_eq!(refilled.pool_spent(pool), -i64::MAX);
            refilled.rollback_state_gas();
            assert_eq!(refilled.pool_remaining(pool), 0);

            let mut caller = GasTracker::new(1_000, 1_000, 0);
            caller.forward_pool(pool, u64::MAX);
            let mut child = GasTracker::new(500, 500, 0);
            assert!(child.record_pool(pool, MAX));
            caller.absorb_returned_pools(&child, true);
            assert_eq!(caller.pool_spent(pool), i64::MAX);
            caller.rollback_state_gas();
            assert_eq!(caller.pool_remaining(pool), u64::MAX);
        }
    }

    /// One unit past it the net would saturate, and a rollback would leave the unlimited pool one
    /// short: a debug build refuses the draw.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "a pool's net leaves the signed range")]
    fn test_a_draw_past_the_signed_maximum_panics_in_a_debug_build() {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        let _ = tracker.record_pool(PoolId::new(2), i64::MAX as u64 + 1);
    }

    /// Below `-i64::MAX` the net would be `i64::MIN`, whose negation saturates, so a share
    /// adopted against it would be one short: a debug build refuses the refill that gets there.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "a pool's net leaves the signed range")]
    fn test_a_refill_past_the_signed_minimum_panics_in_a_debug_build() {
        let pool = PoolId::new(1);
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        tracker.set_pool_remaining(pool, 0);
        tracker.refill_pool(pool, i64::MAX as u64);
        tracker.refill_pool(pool, 1);
    }

    /// With the default pools, forwarding every pool whole and settling a child's return brings
    /// the caller back to each pool less what the child drew and kept: a failed child's draws
    /// come back whole.
    #[test]
    fn test_unlimited_pools_forwarded_whole_come_back_less_the_childs_draws() {
        for fails in [false, true] {
            let mut parent = GasTracker::new(1_000, 1_000, 0);
            let shares = parent.pools_remaining();
            for pool in PoolId::ALL {
                parent.forward_pool(pool, shares.get(pool));
            }
            assert_eq!(parent.pools_remaining(), PoolAmounts::ZERO);

            let mut child = GasTracker::new(500, 500, 0);
            child.set_pools_remaining(shares);
            for pool in PoolId::ALL {
                assert!(child.record_pool(pool, 100));
            }
            if fails {
                child.rollback_state_gas();
            }
            parent.absorb_returned_pools(&child, !fails);

            let (remaining, spent) = if fails {
                (u64::MAX, 0)
            } else {
                (u64::MAX - 100, 100)
            };
            for pool in PoolId::ALL {
                assert_eq!(parent.pool_remaining(pool), remaining, "fails: {fails}");
                assert_eq!(parent.pool_spent(pool), spent, "fails: {fails}");
            }
        }
    }

    /// JSON encodings made before the pools existed decode with unlimited pools, nothing drawn and
    /// no record; the pools' own encoding round-trips.
    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_round_trip_and_older_json_payloads() {
        let mut tracker = pools_1000();
        assert!(tracker.record_pool(PoolId::new(0), 300));
        assert!(!tracker.record_pool(PoolId::new(2), 1_800));
        let encoded = serde_json::to_value(tracker).unwrap();
        assert_eq!(
            serde_json::from_value::<GasTracker>(encoded.clone()).unwrap(),
            tracker
        );

        let mut older = encoded;
        let fields = older.as_object_mut().unwrap();
        fields.remove("pools_remaining");
        fields.remove("pools_spent");
        fields.remove("pool_crossings");
        let decoded = serde_json::from_value::<GasTracker>(older).unwrap();
        assert_eq!(decoded.pools_remaining(), PoolAmounts::UNLIMITED);
        for pool in PoolId::ALL {
            assert_eq!(decoded.pool_spent(pool), 0);
            assert!(!decoded.pool_crossing(pool));
        }
    }
}
