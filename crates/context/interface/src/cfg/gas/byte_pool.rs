//! The byte pool of a [`GasTracker`]: a budget of bytes a frame may append, forwarded to a child
//! frame and settled with the frame's gas.
//!
//! The pool is a figure beside the gas: `bytes_remaining` is what the frame may still draw, and
//! `bytes_spent` is the frame's net draw so far, signed as the state and history nets are. A draw
//! ([`record_bytes`](GasTracker::record_bytes)) the pool cannot pay fails, takes nothing and leaves
//! a [`BytesCrossing`]. A frame that reverts or halts gives its draws back
//! ([`rollback_state_gas`](GasTracker::rollback_state_gas)), and a frame that succeeds hands its
//! net to its caller, so a caller's rollback undoes its children's draws too. The pool starts
//! unlimited (`u64::MAX`), and nothing in this workspace draws it, so a consumer that sets no pool
//! sees nothing of it.

use super::{saturating_signed, GasTracker};
use core::num::NonZeroU64;

/// A draw of the byte pool that the pool could not pay.
///
/// Recorded by [`GasTracker::record_bytes`] when `cost > bytes_remaining`. The failed draw took
/// nothing, so the pool is as it was before it; the record says that a draw failed, and what it
/// asked for. A consumer reads it from a halted frame's result to tell a frame that ran out of
/// bytes from one that ran out of gas, and turns it into a stop of its own class.
///
/// A draw of zero never fails, so the cost is never zero and `Option<BytesCrossing>` takes no more
/// room than a `u64`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BytesCrossing {
    /// The bytes the failed draw asked for.
    cost: NonZeroU64,
}

impl BytesCrossing {
    /// A record of a failed draw of `cost` bytes.
    ///
    /// The tracker makes its own records. This is for a consumer that marks a result it produced
    /// outside the tracker as a failed draw, and puts the record on the tracker with
    /// [`GasTracker::set_bytes_crossing`].
    #[inline]
    pub const fn with_cost(cost: NonZeroU64) -> Self {
        Self { cost }
    }

    /// The bytes the failed draw asked for. Never zero.
    #[inline]
    pub const fn cost(&self) -> u64 {
        self.cost.get()
    }
}

impl GasTracker {
    /// The byte pool: what this frame may still draw.
    ///
    /// Starts at `u64::MAX`, an unlimited pool, in every tracker. A consumer that limits bytes
    /// sets the transaction's pool ([`set_bytes_remaining`](Self::set_bytes_remaining)); every
    /// child frame then receives a share of its caller's pool when it starts, and the caller keeps
    /// the rest.
    #[inline]
    pub const fn bytes_remaining(&self) -> u64 {
        self.bytes_remaining
    }

    /// Sets the byte pool. The net draw ([`bytes_spent`](Self::bytes_spent)) is left as it is.
    #[inline]
    pub const fn set_bytes_remaining(&mut self, val: u64) {
        self.bytes_remaining = val;
    }

    /// The frame's net draw of the pool so far: draws less refills, plus the nets of the children
    /// that succeeded.
    ///
    /// Negative while the frame has refilled more than it drew, as
    /// [`history_gas_spent`](Self::history_gas_spent) can be. A rollback adds it back to the pool.
    #[inline]
    pub const fn bytes_spent(&self) -> i64 {
        self.bytes_spent
    }

    /// Sets the net draw.
    #[inline]
    pub const fn set_bytes_spent(&mut self, val: i64) {
        self.bytes_spent = val;
    }

    /// Adds `delta` to the net draw, saturating. Used to merge a successful child's net into its
    /// caller, so a later rollback of the caller undoes the child's draws too.
    #[inline]
    pub const fn add_bytes_spent(&mut self, delta: i64) {
        self.bytes_spent = self.bytes_spent.saturating_add(delta);
    }

    /// Draws `cost` bytes from the pool.
    ///
    /// Returns `false` if the pool cannot pay, leaving it untouched and recording a
    /// [`BytesCrossing`] ([`bytes_crossing`](Self::bytes_crossing)). A draw of zero always
    /// succeeds. The draw is counted on [`bytes_spent`](Self::bytes_spent), so a rollback gives
    /// it back.
    #[inline]
    #[must_use = "a draw the pool cannot pay is the consumer's stop"]
    pub const fn record_bytes(&mut self, cost: u64) -> bool {
        if let Some(left) = self.bytes_remaining.checked_sub(cost) {
            self.bytes_remaining = left;
            self.bytes_spent = self.bytes_spent.saturating_add(saturating_signed(cost));
            return true;
        }
        self.bytes_draw_failed(cost);
        false
    }

    /// Gives `amount` bytes back to the pool, for a draw the frame took back.
    ///
    /// The net draw falls by the same amount, below zero when the matching draw was a caller's,
    /// which the merge on return reconciles, as [`refill_history`](Self::refill_history) does.
    #[inline]
    pub const fn refill_bytes(&mut self, amount: u64) {
        self.bytes_remaining = self.bytes_remaining.saturating_add(amount);
        self.bytes_spent = self.bytes_spent.saturating_sub(saturating_signed(amount));
    }

    /// Takes `share` out of the pool for a child frame, which starts with it as its pool.
    ///
    /// Not a draw: the net is unchanged, since the share comes back when the child returns
    /// ([`absorb_returned_bytes`](Self::absorb_returned_bytes)). `share` is at most the pool,
    /// which the schedule's forward share guarantees; more saturates at an empty pool.
    #[inline]
    pub const fn forward_bytes(&mut self, share: u64) {
        self.bytes_remaining = self.bytes_remaining.saturating_sub(share);
    }

    /// Adds back what a returning child left of its pool: its remaining pool when it succeeded,
    /// the whole share it was given when it reverted or halted, since its rollback restored it.
    #[inline]
    pub const fn absorb_returned_bytes(&mut self, remaining: u64) {
        self.bytes_remaining = self.bytes_remaining.saturating_add(remaining);
    }

    /// The record of the last draw the pool could not pay, since the record was last cleared, or
    /// one a consumer set with [`set_bytes_crossing`](Self::set_bytes_crossing); whichever was
    /// written last.
    ///
    /// The failed draw took nothing, and no halt touches the pool, so the pool itself still reads
    /// as it was at the failure until the frame's rollback. The record survives
    /// [`spend_all`](Self::spend_all), starts empty in every child, and is not taken over by a
    /// caller.
    #[inline]
    pub const fn bytes_crossing(&self) -> Option<BytesCrossing> {
        self.bytes_crossing
    }

    /// Clears the record returned by [`bytes_crossing`](Self::bytes_crossing).
    #[inline]
    pub const fn clear_bytes_crossing(&mut self) {
        self.bytes_crossing = None;
    }

    /// Sets the record returned by [`bytes_crossing`](Self::bytes_crossing), replacing any record
    /// already there; `None` clears it. The pool is left as it is.
    ///
    /// For a consumer that marks a result it produced itself, such as an answer it gave without
    /// running a frame, as a failed draw, so a classifier reading the record treats it as one the
    /// tracker recorded.
    #[inline]
    pub const fn set_bytes_crossing(&mut self, crossing: Option<BytesCrossing>) {
        self.bytes_crossing = crossing;
    }

    /// Gives the frame's net draw back to the pool, which then reads as the pool the frame
    /// started with, and zeroes the net. Part of [`rollback_state_gas`](Self::rollback_state_gas).
    #[inline]
    pub(super) const fn rollback_bytes(&mut self) {
        self.bytes_remaining = self.bytes_remaining.saturating_add_signed(self.bytes_spent);
        self.bytes_spent = 0;
    }

    /// Records a draw of `cost` the pool could not pay. Only called on the failure side of
    /// [`record_bytes`](Self::record_bytes), where `cost > bytes_remaining >= 0`, so the cost is
    /// never zero and the success side does not pay for the record.
    #[cold]
    #[inline(never)]
    const fn bytes_draw_failed(&mut self, cost: u64) {
        if let Some(cost) = NonZeroU64::new(cost) {
            self.bytes_crossing = Some(BytesCrossing::with_cost(cost));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BytesCrossing, GasTracker};
    use core::num::NonZeroU64;

    /// A tracker with a pool of 1,000 bytes.
    fn pool_1000() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 1_000, 300);
        tracker.set_bytes_remaining(1_000);
        tracker
    }

    fn crossing(tracker: &GasTracker) -> Option<u64> {
        tracker.bytes_crossing().map(|crossing| crossing.cost())
    }

    /// Every tracker starts with an unlimited pool, nothing drawn and no record; `Default` is
    /// `new(0, 0, 0)`.
    #[test]
    fn test_a_new_tracker_has_an_unlimited_pool() {
        let tracker = GasTracker::new(1_000, 1_000, 300);
        assert_eq!(tracker.bytes_remaining(), u64::MAX);
        assert_eq!(tracker.bytes_spent(), 0);
        assert_eq!(tracker.bytes_crossing(), None);
        assert_eq!(GasTracker::default(), GasTracker::new(0, 0, 0));
        assert_eq!(GasTracker::default().bytes_remaining(), u64::MAX);
    }

    /// An absent record takes no extra room.
    #[test]
    fn test_an_absent_crossing_takes_no_extra_room() {
        assert_eq!(
            core::mem::size_of::<Option<BytesCrossing>>(),
            core::mem::size_of::<u64>()
        );
    }

    #[test]
    fn test_record_bytes_draws_the_pool_and_counts_the_draw() {
        let mut tracker = pool_1000();

        assert!(tracker.record_bytes(300));
        assert!(tracker.record_bytes(200));

        assert_eq!(tracker.bytes_remaining(), 500);
        assert_eq!(tracker.bytes_spent(), 500);
        assert_eq!(tracker.bytes_crossing(), None);
    }

    /// A draw the pool cannot pay takes nothing and leaves a record of what it asked for.
    #[test]
    fn test_a_draw_beyond_the_pool_fails_and_records_the_crossing() {
        let mut tracker = pool_1000();
        assert!(tracker.record_bytes(900));

        assert!(!tracker.record_bytes(101));

        assert_eq!(
            tracker.bytes_remaining(),
            100,
            "the failed draw took nothing"
        );
        assert_eq!(tracker.bytes_spent(), 900);
        assert_eq!(crossing(&tracker), Some(101));
    }

    /// A draw of exactly the pool succeeds, and a draw of zero succeeds on an empty pool.
    #[test]
    fn test_a_draw_of_the_whole_pool_succeeds_and_zero_always_does() {
        let mut tracker = pool_1000();
        assert!(tracker.record_bytes(1_000));
        assert_eq!(tracker.bytes_remaining(), 0);
        assert!(tracker.record_bytes(0));
        assert_eq!(tracker.bytes_crossing(), None);
        assert!(!tracker.record_bytes(1));
        assert_eq!(crossing(&tracker), Some(1));
    }

    /// The record stays until it is cleared: a later draw that succeeds leaves it, a later failed
    /// draw replaces it, and a consumer clears or sets it.
    #[test]
    fn test_the_record_stays_until_cleared_and_the_last_failure_wins() {
        let mut tracker = pool_1000();
        assert!(!tracker.record_bytes(1_001));
        assert!(tracker.record_bytes(10));
        assert_eq!(crossing(&tracker), Some(1_001));

        assert!(!tracker.record_bytes(2_000));
        assert_eq!(crossing(&tracker), Some(2_000));

        tracker.clear_bytes_crossing();
        assert_eq!(tracker.bytes_crossing(), None);

        let set = BytesCrossing::with_cost(NonZeroU64::new(7).unwrap());
        tracker.set_bytes_crossing(Some(set));
        assert_eq!(tracker.bytes_crossing(), Some(set));
        assert_eq!(
            tracker.bytes_remaining(),
            990,
            "setting the record leaves the pool"
        );
    }

    /// The halt that follows a failed draw zeroes the regular gas and nothing else: the record
    /// and the pool are still readable after it.
    #[test]
    fn test_the_record_and_the_pool_survive_spend_all() {
        let mut tracker = pool_1000();
        assert!(tracker.record_bytes(600));
        assert!(!tracker.record_bytes(500));

        tracker.spend_all();

        assert_eq!(tracker.remaining(), 0);
        assert_eq!(tracker.bytes_remaining(), 400);
        assert_eq!(crossing(&tracker), Some(500));
    }

    /// A refill gives bytes back and lowers the net, below zero when the draw was a caller's.
    #[test]
    fn test_refill_bytes_gives_bytes_back_and_can_take_the_net_below_zero() {
        let mut tracker = pool_1000();
        assert!(tracker.record_bytes(300));

        tracker.refill_bytes(500);

        assert_eq!(tracker.bytes_remaining(), 1_200);
        assert_eq!(tracker.bytes_spent(), -200);
    }

    /// Rolling a frame back gives its net draw back, so the pool reads as the pool the frame
    /// started with, and nothing else of the rollback changes.
    #[test]
    fn test_rollback_restores_the_pool_the_frame_started_with() {
        let mut tracker = pool_1000();
        assert!(tracker.record_state_cost(200));
        assert!(tracker.record_bytes(300));
        tracker.refill_bytes(100);
        assert!(tracker.record_bytes(50));
        assert_eq!(tracker.bytes_remaining(), 750);

        tracker.rollback_state_gas();

        assert_eq!(tracker.bytes_remaining(), 1_000);
        assert_eq!(tracker.bytes_spent(), 0);
        assert_eq!(tracker.reservoir(), 300, "the state rollback is as it was");
        assert_eq!(tracker.state_gas_spent(), 0);
    }

    /// A rollback of a frame that refilled a caller's draw takes the refill back out.
    #[test]
    fn test_rollback_undoes_a_refill_of_a_callers_draw() {
        let mut tracker = pool_1000();
        tracker.refill_bytes(100);
        assert_eq!(tracker.bytes_remaining(), 1_100);

        tracker.rollback_state_gas();

        assert_eq!(tracker.bytes_remaining(), 1_000);
        assert_eq!(tracker.bytes_spent(), 0);
    }

    /// The forward takes the share out of the pool without counting it as a draw; the child's
    /// return adds back what it left.
    #[test]
    fn test_forward_takes_the_share_and_the_return_adds_back_what_the_child_left() {
        let mut parent = pool_1000();
        parent.forward_bytes(980);
        assert_eq!(parent.bytes_remaining(), 20);
        assert_eq!(parent.bytes_spent(), 0, "a forward is not a draw");

        parent.absorb_returned_bytes(700);
        assert_eq!(parent.bytes_remaining(), 720);
    }

    /// A forward above the pool and a return above `u64::MAX` saturate rather than wrap.
    #[test]
    fn test_forward_and_return_saturate() {
        let mut tracker = pool_1000();
        tracker.forward_bytes(5_000);
        assert_eq!(tracker.bytes_remaining(), 0);

        let mut unlimited = GasTracker::new(1, 1, 0);
        unlimited.absorb_returned_bytes(1);
        assert_eq!(unlimited.bytes_remaining(), u64::MAX);
    }

    /// With the default pool, forwarding the whole pool and settling a child's return brings the
    /// caller back to the pool less what the child drew and kept: a failed child's draws come
    /// back whole.
    #[test]
    fn test_an_unlimited_pool_forwarded_whole_comes_back_less_the_childs_draws() {
        for fails in [false, true] {
            let mut parent = GasTracker::new(1_000, 1_000, 0);
            let share = parent.bytes_remaining();
            parent.forward_bytes(share);
            assert_eq!(parent.bytes_remaining(), 0);

            let mut child = GasTracker::new(500, 500, 0);
            child.set_bytes_remaining(share);
            assert!(child.record_bytes(100));
            if fails {
                child.rollback_state_gas();
            } else {
                parent.add_bytes_spent(child.bytes_spent());
            }
            parent.absorb_returned_bytes(child.bytes_remaining());

            let (pool, spent) = if fails {
                (u64::MAX, 0)
            } else {
                (u64::MAX - 100, 100)
            };
            assert_eq!(parent.bytes_remaining(), pool, "fails: {fails}");
            assert_eq!(parent.bytes_spent(), spent, "fails: {fails}");
        }
    }

    /// Encodings made before the pool existed decode with an unlimited pool, nothing drawn and no
    /// record; the pool's own encoding round-trips.
    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_round_trip_and_older_payloads() {
        let mut tracker = pool_1000();
        assert!(tracker.record_bytes(300));
        assert!(!tracker.record_bytes(800));
        let encoded = serde_json::to_value(tracker).unwrap();
        assert_eq!(
            serde_json::from_value::<GasTracker>(encoded.clone()).unwrap(),
            tracker
        );

        let mut older = encoded;
        let fields = older.as_object_mut().unwrap();
        fields.remove("bytes_remaining");
        fields.remove("bytes_spent");
        fields.remove("bytes_crossing");
        let decoded = serde_json::from_value::<GasTracker>(older).unwrap();
        assert_eq!(decoded.bytes_remaining(), u64::MAX);
        assert_eq!(decoded.bytes_spent(), 0);
        assert_eq!(decoded.bytes_crossing(), None);
    }
}
