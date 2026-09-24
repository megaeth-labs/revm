//! Gas constants and functions for gas calculation.

use crate::{cfg::gas_params, cfg::GasParams, Transaction};
use primitives::hardfork::SpecId;

mod withheld;

pub use withheld::WithheldCrossing;

/// Tracker for gas during execution.
///
/// This is used to track the gas during execution.
///
/// Regular gas is held in two parts: the spendable part ([`spendable`](Self::spendable)), the
/// only part a regular charge draws, and the withheld part ([`withheld`](Self::withheld)), which
/// a consumer can hold back from regular charges with [`withhold`](Self::withhold).
/// [`remaining`](Self::remaining) is their sum, and it is what every other reader of the frame's
/// gas sees: `GAS`, the gas forwarded to a child frame, the `SSTORE` stipend sentry, the
/// skip-cold-load checks, the gas a child returns to its parent and the post-execution
/// reimbursement. Deductions that are not regular charges draw the withheld part first
/// ([`record_withheld_first_cost`](Self::record_withheld_first_cost)); credits of regular gas land
/// on the spendable part, and a consumer that wants to keep gas held back withholds again after
/// them. A regular charge the withheld part would have paid fails as it would with nothing
/// withheld, and leaves behind a [`WithheldCrossing`] that holds the withheld part at the charge.
/// With nothing withheld, which is the default, every method behaves as it did before the
/// withheld part existed.
///
/// The net counters (`state_gas_spent`, `history_gas_spent`) are `i64`, while charges and refills
/// take `u64` amounts and convert them with saturation. The tracker assumes that the transaction
/// gas limit, and so any single charge or refill and the state plus history net a rollback adds
/// back, is at most `i64::MAX`. Upstream's state gas accounting assumes the same bound
/// ([`record_state_cost`](Self::record_state_cost) converts the charge with `as i64`). Nothing
/// checks it. Above it, a rollback no longer restores the reservoir exactly and can burn or mint
/// gas at the boundary. Reaching it takes a transaction gas limit above `i64::MAX`
/// (about 9.2 × 10^18).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GasTracker {
    /// Gas Limit,
    gas_limit: u64,
    /// Regular gas remaining (`gas_left`). Reservoir is tracked separately.
    ///
    /// This is the spendable part: gas withheld from regular charges is tracked separately in
    /// `withheld`, and [`remaining`](Self::remaining) reports the sum of the two.
    remaining: u64,
    /// State gas reservoir (gas exceeding TX_MAX_GAS_LIMIT). Starts as `execution_gas - min(execution_gas, regular_gas_budget)`.
    /// When 0, all remaining gas is regular gas with hard cap at `TX_MAX_GAS_LIMIT`.
    reservoir: u64,
    /// Net state gas spent so far.
    ///
    /// Can be negative within a call frame when 0→x→0 storage restoration refills
    /// more state gas than the frame itself has charged (the parent previously
    /// charged the 0→x portion). The net is reconciled on frame return.
    state_gas_spent: i64,
    /// State gas drawn from regular gas (`remaining`) because the reservoir was
    /// empty (EIP-8037's `state_gas_from_gas_left`).
    ///
    /// Incremented by [`Self::record_state_cost`] whenever a state-gas charge
    /// spills out of the reservoir into regular gas. On frame rollback (revert or
    /// halt) the spilled portion is credited back to `remaining` in last-in-
    /// first-out order by [`Self::rollback_state_gas`]; on success it is
    /// propagated to the parent frame so a later parent rollback can return it.
    state_gas_spilled: u64,
    /// Net history gas spent so far.
    ///
    /// History gas pays for the bytes a transaction appends to the chain's history — log
    /// records, deployed code, the transaction body — as opposed to the bytes it adds to the
    /// world state. It draws on the same reservoir-first budget as state gas and unwinds on
    /// the same paths, but is counted apart so the two dimensions can be reported and limited
    /// separately.
    ///
    /// Signed for the same reason as `state_gas_spent`: a frame can refill history gas a parent
    /// frame charged ([`refill_history`](Self::refill_history)), which leaves the frame's own net
    /// below zero until it merges into that parent.
    ///
    /// Stays zero unless [`record_history_cost`](Self::record_history_cost) or
    /// [`refill_history`](Self::refill_history) is called. The only caller in this workspace is the
    /// code-deposit charge in `return_create`, which charges nothing unless the schedule gives
    /// `code_deposit_history_gas` a price, so a chain that does not price history bytes behaves
    /// exactly as it did before this field existed.
    ///
    /// Encodings made before this field existed decode it as zero.
    #[cfg_attr(feature = "serde", serde(default))]
    history_gas_spent: i64,
    /// Refunded gas. Used to refund the gas to the caller at the end of execution.
    refunded: i64,
    /// Regular gas withheld from regular charges ([`withhold`](Self::withhold)).
    ///
    /// Counted by [`remaining`](Self::remaining) and drawn first by deductions that are not
    /// regular charges ([`record_withheld_first_cost`](Self::record_withheld_first_cost)), but
    /// never by a regular charge. Zero unless a consumer withholds gas; while it is zero, every
    /// method behaves as it did before this field existed.
    ///
    /// Encodings made before this field existed decode it as zero.
    #[cfg_attr(feature = "serde", serde(default))]
    withheld: u64,
    /// The record of the last failed regular charge the withheld part would have paid
    /// ([`withheld_crossing`](Self::withheld_crossing)): the withheld part at that charge. One
    /// word, since the withheld part in a crossing is never zero.
    ///
    /// Encodings made before this field existed decode it as `None`.
    #[cfg_attr(feature = "serde", serde(default))]
    withheld_crossing: Option<WithheldCrossing>,
}

impl GasTracker {
    /// Creates a new `GasTracker` with the given remaining gas and reservoir.
    #[inline]
    pub const fn new(gas_limit: u64, remaining: u64, reservoir: u64) -> Self {
        Self {
            gas_limit,
            remaining,
            reservoir,
            state_gas_spent: 0,
            state_gas_spilled: 0,
            history_gas_spent: 0,
            refunded: 0,
            withheld: 0,
            withheld_crossing: None,
        }
    }

    /// Creates a new `GasTracker` with the given used gas and reservoir.
    /// Remaining gas saturates at zero when used gas exceeds the gas limit.
    #[inline]
    pub const fn new_used_gas(gas_limit: u64, used_gas: u64, reservoir: u64) -> Self {
        Self::new(gas_limit, gas_limit.saturating_sub(used_gas), reservoir)
    }

    /// Returns the gas limit.
    #[inline]
    pub const fn limit(&self) -> u64 {
        self.gas_limit
    }

    /// Sets the gas limit.
    #[inline]
    pub const fn set_limit(&mut self, val: u64) {
        self.gas_limit = val;
    }

    /// Returns the remaining gas: the spendable part plus the withheld part.
    ///
    /// Every reader of the frame's gas sees this total. Only a regular charge is limited to
    /// the spendable part ([`spendable`](Self::spendable)). With nothing withheld the two are
    /// equal.
    ///
    /// The sum wraps instead of overflowing. It can only exceed `u64::MAX` after a failed
    /// [`record_cost_unsafe`](Self::record_cost_unsafe), which wraps the spendable part and
    /// leaves the frame to halt; the total then reads as if the failed charge had been taken
    /// from it.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.remaining.wrapping_add(self.withheld)
    }

    /// Sets the remaining gas, the total [`remaining`](Self::remaining) returns.
    ///
    /// The withheld part is kept as far as the new total allows: it becomes
    /// `min(withheld, val)` and the spendable part takes the rest. `remaining()` then returns
    /// `val`, and no gas is withheld that was not withheld before. With nothing withheld this
    /// sets the spendable part to `val`.
    #[inline]
    pub const fn set_remaining(&mut self, val: u64) {
        if self.withheld > val {
            self.withheld = val;
        }
        self.remaining = val - self.withheld;
    }

    /// Returns the reservoir gas.
    #[inline]
    pub const fn reservoir(&self) -> u64 {
        self.reservoir
    }

    /// Sets the reservoir gas.
    #[inline]
    pub const fn set_reservoir(&mut self, val: u64) {
        self.reservoir = val;
    }

    /// Adopts a reservoir returned by a child frame and reconciles it with any
    /// outstanding state gas spilled into regular gas.
    ///
    /// A successful child can refill state gas charged by an ancestor (for
    /// example, by clearing a slot created by a sibling). Since the child does
    /// not inherit the ancestor's [`Self::state_gas_spilled`] counter, that
    /// refill initially lands in the child's reservoir. On return it must first
    /// restore the parent's regular gas in last-in-first-out order; only the
    /// excess remains in the reservoir.
    ///
    /// This only reconciles the funding pools. The child's signed
    /// `state_gas_spent` has already accounted for the refill and is merged
    /// separately by the frame handler.
    #[inline]
    pub const fn absorb_returned_reservoir(&mut self, reservoir: u64) {
        let to_remaining = if reservoir < self.state_gas_spilled {
            reservoir
        } else {
            self.state_gas_spilled
        };
        self.remaining = self.remaining.saturating_add(to_remaining);
        self.state_gas_spilled -= to_remaining;
        self.reservoir = reservoir - to_remaining;
    }

    /// Returns the state gas spent.
    #[inline]
    pub const fn state_gas_spent(&self) -> i64 {
        self.state_gas_spent
    }

    /// Sets the state gas spent.
    #[inline]
    pub const fn set_state_gas_spent(&mut self, val: i64) {
        self.state_gas_spent = val;
    }

    /// Returns the state gas drawn from regular gas (`remaining`) because the
    /// reservoir was empty (EIP-8037's `state_gas_from_gas_left`).
    #[inline]
    pub const fn state_gas_spilled(&self) -> u64 {
        self.state_gas_spilled
    }

    /// Sets the spilled state gas.
    #[inline]
    pub const fn set_state_gas_spilled(&mut self, val: u64) {
        self.state_gas_spilled = val;
    }

    /// Adds `delta` to the spilled state gas, saturating.
    ///
    /// Used to merge a successful child frame's spilled state gas into this
    /// (parent) frame so a later parent rollback can return it.
    #[inline]
    pub const fn add_state_gas_spilled(&mut self, delta: u64) {
        self.state_gas_spilled = self.state_gas_spilled.saturating_add(delta);
    }

    /// Returns the history gas spent.
    ///
    /// Negative while this frame has refilled more history gas than it charged, like
    /// [`state_gas_spent`](Self::state_gas_spent).
    #[inline]
    pub const fn history_gas_spent(&self) -> i64 {
        self.history_gas_spent
    }

    /// Sets the history gas spent.
    #[inline]
    pub const fn set_history_gas_spent(&mut self, val: i64) {
        self.history_gas_spent = val;
    }

    /// Returns the refunded gas.
    #[inline]
    pub const fn refunded(&self) -> i64 {
        self.refunded
    }

    /// Sets the refunded gas.
    #[inline]
    pub const fn set_refunded(&mut self, val: i64) {
        self.refunded = val;
    }

    /// Records a regular gas cost.
    ///
    /// Deducts from the spendable part of `remaining`; the withheld part cannot pay a regular
    /// charge. Returns `false` if insufficient gas, leaving the tracker's gas untouched. If the
    /// withheld part would have covered the charge, the failure is first recorded as a
    /// [`WithheldCrossing`].
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_regular_cost(&mut self, cost: u64) -> bool {
        if let Some(new_remaining) = self.remaining.checked_sub(cost) {
            self.remaining = new_remaining;
            return true;
        }
        self.regular_charge_failed(cost, self.remaining);
        false
    }

    /// Records a regular gas cost without bounds checking.
    ///
    /// Deducts from the spendable part, wrapping on underflow, and returns `true` if it could
    /// not pay. The caller must then halt the frame out of gas, which zeroes the regular gas
    /// ([`spend_all`](Self::spend_all)). A failure the withheld part would have covered is
    /// recorded as a [`WithheldCrossing`], as in
    /// [`record_regular_cost`](Self::record_regular_cost).
    #[inline(always)]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_cost_unsafe(&mut self, cost: u64) -> bool {
        let remaining = self.remaining;
        let oog = remaining < cost;
        self.remaining = remaining.wrapping_sub(cost);
        if oog {
            self.regular_charge_failed(cost, remaining);
        }
        oog
    }

    /// Records a state gas cost (EIP-8037 reservoir model).
    ///
    /// State gas charges deduct from the reservoir first. If the reservoir is exhausted,
    /// remaining charges spill into `remaining` (requiring `remaining >= cost`), withheld part
    /// first ([`record_withheld_first_cost`](Self::record_withheld_first_cost)).
    /// Tracks state gas spent.
    ///
    /// Returns `false` if total remaining gas is insufficient.
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_state_cost(&mut self, cost: u64) -> bool {
        if self.reservoir >= cost {
            self.state_gas_spent = self.state_gas_spent.saturating_add(cost as i64);
            self.reservoir -= cost;
            return true;
        }

        let spill = cost - self.reservoir;

        let success = self.record_withheld_first_cost(spill);
        if success {
            self.state_gas_spent = self.state_gas_spent.saturating_add(cost as i64);
            self.state_gas_spilled = self.state_gas_spilled.saturating_add(spill);
            self.reservoir = 0;
        }
        success
    }

    /// Records a history gas cost, drawn from the same budget as a state-gas charge.
    ///
    /// The charge deducts from the reservoir first and spills into `remaining` (withheld part
    /// first) once the reservoir is exhausted, exactly as
    /// [`record_state_cost`](Self::record_state_cost) does, and the spilled portion joins
    /// `state_gas_spilled` so a rollback credits it back to `remaining` in the same
    /// last-in-first-out order. What differs is the counter: the amount lands in
    /// `history_gas_spent`, which is what keeps the history dimension separable from the state
    /// dimension downstream.
    ///
    /// Returns `false` if the total remaining budget is insufficient, leaving the tracker
    /// untouched.
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_history_cost(&mut self, cost: u64) -> bool {
        if self.reservoir >= cost {
            self.history_gas_spent = self
                .history_gas_spent
                .saturating_add(saturating_signed(cost));
            self.reservoir -= cost;
            return true;
        }

        let spill = cost - self.reservoir;

        let success = self.record_withheld_first_cost(spill);
        if success {
            self.history_gas_spent = self
                .history_gas_spent
                .saturating_add(saturating_signed(cost));
            self.state_gas_spilled = self.state_gas_spilled.saturating_add(spill);
            self.reservoir = 0;
        }
        success
    }

    /// Refills history gas for bytes a frame charged for and then took back, such as a storage
    /// write restored to its original value within the transaction.
    ///
    /// The history-gas counterpart of [`refill_reservoir`](Self::refill_reservoir), with the same
    /// last-in-first-out order: `remaining` is credited up to `state_gas_spilled`, the rest tops up
    /// the reservoir, and the net history gas falls by the full `amount` — below zero when the
    /// matching charge was made by a parent frame, which the merge on return reconciles.
    #[inline]
    pub const fn refill_history(&mut self, amount: u64) {
        let to_remaining = if amount < self.state_gas_spilled {
            amount
        } else {
            self.state_gas_spilled
        };
        self.remaining = self.remaining.saturating_add(to_remaining);
        self.state_gas_spilled -= to_remaining;
        self.reservoir = self.reservoir.saturating_add(amount - to_remaining);
        self.history_gas_spent = self
            .history_gas_spent
            .saturating_sub(saturating_signed(amount));
    }

    /// Rolls back this frame's state-gas and history-gas charges on revert or
    /// exceptional halt (EIP-8037).
    ///
    /// Everything the frame charged against the reservoir-first budget is refilled in
    /// last-in-first-out order: the spilled portion is credited back to `remaining` (the pool
    /// charged last) and the rest restores the reservoir to its frame-start value. Concretely,
    /// `remaining` gains `state_gas_spilled` and the reservoir becomes
    /// `reservoir + state_gas_spent + history_gas_spent - state_gas_spilled`, which is exactly
    /// the reservoir the frame inherited. All three counters are then reset.
    ///
    /// History gas is part of the same unwind because it comes out of the same two pools and
    /// pays for bytes the failing frame no longer appends. The term is zero on any chain that
    /// never calls [`record_history_cost`](Self::record_history_cost).
    ///
    /// On revert the resulting `remaining` (including the refilled spill) is
    /// returned to the parent; on halt the caller additionally zeroes `remaining`
    /// so the spilled gas is consumed while the reservoir is left untouched.
    #[inline]
    pub const fn rollback_state_gas(&mut self) {
        // The two nets are summed before they meet the reservoir: a frame that took back a parent's
        // state charge after its own history charge spilled holds less reservoir than its state
        // net is negative, and adding the terms one at a time would clamp at zero part-way.
        let charged = self.state_gas_spent.saturating_add(self.history_gas_spent);
        self.reservoir = self
            .reservoir
            .saturating_add_signed(charged)
            .saturating_sub(self.state_gas_spilled);
        self.remaining = self.remaining.saturating_add(self.state_gas_spilled);
        self.state_gas_spent = 0;
        self.state_gas_spilled = 0;
        self.history_gas_spent = 0;
    }

    /// Refills the reservoir with state gas that is returned by 0→x→0 storage
    /// restoration (EIP-8037 issue #2).
    ///
    /// Per the spec, when a storage slot is restored to its original zero value
    /// within the same transaction, the state gas charged for the initial 0→x
    /// transition is directly restored to the reservoir rather than routed
    /// through the capped refund counter.
    ///
    /// `state_gas_spent` is decremented by the full `amount` and may become
    /// negative if the matching 0→x charge was made by a parent frame (so this
    /// frame's `state_gas_spilled` is zero and the whole refill lands in the
    /// reservoir); the parent's total is reconciled on frame return.
    ///
    /// Because charges deduct from the reservoir first and from regular gas
    /// (`remaining`) last, the refill credits the pool charged last first:
    /// `remaining` is credited up to `state_gas_spilled` and any remainder tops
    /// up the reservoir.
    #[inline]
    pub const fn refill_reservoir(&mut self, amount: u64) {
        let to_remaining = if amount < self.state_gas_spilled {
            amount
        } else {
            self.state_gas_spilled
        };
        self.remaining = self.remaining.saturating_add(to_remaining);
        self.state_gas_spilled -= to_remaining;
        self.reservoir = self.reservoir.saturating_add(amount - to_remaining);
        self.state_gas_spent = self.state_gas_spent.saturating_sub(amount as i64);
    }

    /// Records a refund value.
    #[inline]
    pub const fn record_refund(&mut self, refund: i64) {
        self.refunded += refund;
    }

    /// Erases a gas cost from remaining (returns gas from child frame).
    ///
    /// The gas lands on the spendable part, as every credit of regular gas does.
    #[inline]
    pub const fn erase_cost(&mut self, returned: u64) {
        self.remaining += returned;
    }

    /// Spends all remaining gas excluding the reservoir: the spendable and the withheld part.
    ///
    /// A recorded [`WithheldCrossing`] is kept, so it can be read after the halt, and with it
    /// the withheld part this zeroes.
    #[inline]
    pub const fn spend_all(&mut self) {
        self.remaining = 0;
        self.withheld = 0;
    }
}

/// `value` as a signed gas amount, saturating at `i64::MAX`.
#[inline]
const fn saturating_signed(value: u64) -> i64 {
    if value > i64::MAX as u64 {
        i64::MAX
    } else {
        value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::GasTracker;

    /// A history charge spends the reservoir before it spends regular gas, and books the
    /// amount on its own counter. Were it booked on `state_gas_spent`, the two dimensions
    /// would be indistinguishable in every downstream report.
    #[test]
    fn test_record_history_cost_draws_the_reservoir_first() {
        let mut tracker = GasTracker::new(1_000, 400, 300);

        assert!(tracker.record_history_cost(200));

        assert_eq!(tracker.reservoir(), 100);
        assert_eq!(tracker.remaining(), 400);
        assert_eq!(tracker.history_gas_spent(), 200);
        assert_eq!(tracker.state_gas_spent(), 0);
        assert_eq!(tracker.state_gas_spilled(), 0);
    }

    /// Once the reservoir is empty the rest of the charge spills onto regular gas, and the
    /// spill is recorded so a rollback knows which pool to credit back first.
    #[test]
    fn test_record_history_cost_spills_onto_regular_gas() {
        let mut tracker = GasTracker::new(1_000, 400, 300);

        assert!(tracker.record_history_cost(500));

        assert_eq!(tracker.reservoir(), 0);
        assert_eq!(tracker.remaining(), 200);
        assert_eq!(tracker.history_gas_spent(), 500);
        assert_eq!(tracker.state_gas_spilled(), 200);
    }

    /// A charge larger than both pools together leaves the tracker untouched, so the caller
    /// can halt without having half-spent the budget.
    #[test]
    fn test_record_history_cost_rejects_what_it_cannot_pay() {
        let mut tracker = GasTracker::new(1_000, 400, 300);

        assert!(!tracker.record_history_cost(701));

        assert_eq!(tracker.reservoir(), 300);
        assert_eq!(tracker.remaining(), 400);
        assert_eq!(tracker.history_gas_spent(), 0);
    }

    /// Rolling back a frame restores the reservoir and regular gas a history charge took,
    /// exactly as it does for a state charge, and both counters come back to zero.
    #[test]
    fn test_rollback_returns_history_gas_to_both_pools() {
        let mut tracker = GasTracker::new(1_000, 400, 300);
        assert!(tracker.record_history_cost(500));

        tracker.rollback_state_gas();

        assert_eq!(
            tracker.reservoir(),
            300,
            "the frame-start reservoir is restored"
        );
        assert_eq!(
            tracker.remaining(),
            400,
            "the spilled portion goes back to regular gas"
        );
        assert_eq!(tracker.history_gas_spent(), 0);
        assert_eq!(tracker.state_gas_spilled(), 0);
    }

    /// State and history charges share one budget and one spill counter, so a rollback that
    /// saw both must still land on the reservoir the frame inherited.
    #[test]
    fn test_rollback_returns_mixed_state_and_history_charges() {
        let mut tracker = GasTracker::new(1_000, 400, 300);
        assert!(tracker.record_state_cost(200));
        assert!(tracker.record_history_cost(300));

        assert_eq!(tracker.reservoir(), 0);
        assert_eq!(tracker.remaining(), 200);

        tracker.rollback_state_gas();

        assert_eq!(tracker.reservoir(), 300);
        assert_eq!(tracker.remaining(), 400);
        assert_eq!(tracker.state_gas_spent(), 0);
        assert_eq!(tracker.history_gas_spent(), 0);
    }

    /// A 0→x→0 refill offsets state gas only. It may consume spill a history charge
    /// contributed — the pools are fungible — but it must never move the history counter.
    #[test]
    fn test_refill_reservoir_leaves_the_history_counter_alone() {
        let mut tracker = GasTracker::new(1_000, 400, 300);
        assert!(tracker.record_history_cost(300));
        assert!(tracker.record_state_cost(250));

        tracker.refill_reservoir(250);

        assert_eq!(tracker.state_gas_spent(), 0);
        assert_eq!(tracker.history_gas_spent(), 300);
    }

    /// A restored write's history is refilled the way a restored slot's state gas is: the spilled
    /// part back to regular gas first, the rest to the reservoir, and the counter back down.
    #[test]
    fn test_refill_history_returns_the_spill_first() {
        let mut tracker = GasTracker::new(1_000, 400, 300);
        assert!(tracker.record_history_cost(500));
        assert_eq!((tracker.reservoir(), tracker.remaining()), (0, 200));

        tracker.refill_history(250);

        assert_eq!(
            tracker.remaining(),
            400,
            "the 200 spilled come back to regular gas"
        );
        assert_eq!(
            tracker.reservoir(),
            50,
            "the other 50 go back to the reservoir"
        );
        assert_eq!(tracker.state_gas_spilled(), 0);
        assert_eq!(tracker.history_gas_spent(), 250);
    }

    /// A child that refills history its parent charged goes below zero, and merging it into the
    /// parent on success nets the two out. The merge follows the handler's order: the signed figure
    /// is read, added and written back, as the state counter's is, and then the parent absorbs the
    /// child's reservoir.
    #[test]
    fn test_a_child_refill_of_a_parent_charge_nets_out_on_merge() {
        let mut parent = GasTracker::new(1_000, 1_000, 500);
        assert!(parent.record_history_cost(100));

        let mut child = GasTracker::new(300, 300, parent.reservoir());
        child.refill_history(100);
        assert_eq!(child.history_gas_spent(), -100);

        parent.set_history_gas_spent(
            parent
                .history_gas_spent()
                .saturating_add(child.history_gas_spent()),
        );
        // The parent's charge came out of the reservoir and spilled nothing, so the child's
        // reservoir has no spill to pay back and the parent keeps all of it.
        parent.absorb_returned_reservoir(child.reservoir());
        assert_eq!(parent.history_gas_spent(), 0);
        assert_eq!(parent.reservoir(), 500);
    }

    /// Copying the counter through its getter and setter, as `PrecompileOutput` does for state gas,
    /// keeps a negative net.
    #[test]
    fn test_a_negative_history_net_survives_a_copy_through_the_accessors() {
        let mut child = GasTracker::new(300, 300, 0);
        child.refill_history(100);

        let mut copy = GasTracker::new(300, 300, child.reservoir());
        copy.set_history_gas_spent(child.history_gas_spent());

        assert_eq!(copy.history_gas_spent(), -100);
        assert_eq!(copy, child);
    }

    /// Rolling back a frame that refilled a parent's history charge takes the refill back out of
    /// the reservoir, landing on the reservoir the frame inherited.
    #[test]
    fn test_rollback_undoes_a_refill_of_a_parent_charge() {
        let mut child = GasTracker::new(300, 300, 400);
        child.refill_history(100);
        assert_eq!(child.reservoir(), 500);

        child.rollback_state_gas();

        assert_eq!(child.reservoir(), 400);
        assert_eq!(child.history_gas_spent(), 0);
    }

    #[test]
    fn new_used_gas_saturates_remaining_at_zero() {
        assert_eq!(
            GasTracker::new_used_gas(10, 9, 3),
            GasTracker::new(10, 1, 3)
        );
        assert_eq!(
            GasTracker::new_used_gas(10, 10, 3),
            GasTracker::new(10, 0, 3)
        );
        assert_eq!(
            GasTracker::new_used_gas(10, 11, 3),
            GasTracker::new(10, 0, 3)
        );
    }

    #[test]
    fn returned_reservoir_restores_spilled_state_gas_first() {
        let mut gas = GasTracker::new(1_000, 600, 0);
        assert!(gas.record_state_cost(400));

        gas.absorb_returned_reservoir(250);

        assert_eq!(gas.remaining(), 450);
        assert_eq!(gas.reservoir(), 0);
        assert_eq!(gas.state_gas_spilled(), 150);
        assert_eq!(gas.state_gas_spent(), 400);

        gas.absorb_returned_reservoir(200);

        assert_eq!(gas.remaining(), 600);
        assert_eq!(gas.reservoir(), 50);
        assert_eq!(gas.state_gas_spilled(), 0);
        assert_eq!(gas.state_gas_spent(), 400);
    }
}

/// Gas cost for operations that consume zero gas.
pub const ZERO: u64 = 0;
/// Base gas cost for basic operations.
pub const BASE: u64 = 2;

/// Gas cost for very low-cost operations.
pub const VERYLOW: u64 = 3;
/// Gas cost for DATALOADN instruction.
pub const DATA_LOADN_GAS: u64 = 3;

/// Gas cost for conditional jump instructions.
pub const CONDITION_JUMP_GAS: u64 = 4;
/// Gas cost for RETF instruction.
pub const RETF_GAS: u64 = 3;
/// Gas cost for DATALOAD instruction.
pub const DATA_LOAD_GAS: u64 = 4;

/// Gas cost for low-cost operations.
pub const LOW: u64 = 5;
/// Gas cost for medium-cost operations.
pub const MID: u64 = 8;
/// Gas cost for high-cost operations.
pub const HIGH: u64 = 10;
/// Gas cost for JUMPDEST instruction.
pub const JUMPDEST: u64 = 1;
/// Gas cost for REFUND SELFDESTRUCT instruction.
pub const SELFDESTRUCT_REFUND: i64 = 24000;
/// Gas cost for CREATE instruction.
pub const CREATE: u64 = 32000;
/// Additional gas cost when a call transfers value.
pub const CALLVALUE: u64 = 9000;
/// Gas cost for creating a new account.
pub const NEWACCOUNT: u64 = 25000;
/// Base gas cost for EXP instruction.
pub const EXP: u64 = 10;
/// Gas cost per word for memory operations.
pub const MEMORY: u64 = 3;
/// Base gas cost for LOG instructions.
pub const LOG: u64 = 375;
/// Gas cost per byte of data in LOG instructions.
pub const LOGDATA: u64 = 8;
/// Gas cost per topic in LOG instructions.
pub const LOGTOPIC: u64 = 375;
/// Base gas cost for KECCAK256 instruction.
pub const KECCAK256: u64 = 30;
/// Gas cost per word for KECCAK256 instruction.
pub const KECCAK256WORD: u64 = 6;
/// Gas cost per word for copy operations.
pub const COPY: u64 = 3;
/// Gas cost for BLOCKHASH instruction.
pub const BLOCKHASH: u64 = 20;
/// Gas cost per byte for code deposit during contract creation.
pub const CODEDEPOSIT: u64 = 200;

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub const ISTANBUL_SLOAD_GAS: u64 = 800;
/// Gas cost for SSTORE when setting a storage slot from zero to non-zero.
pub const SSTORE_SET: u64 = 20000;
/// Gas cost for SSTORE when modifying an existing non-zero storage slot.
pub const SSTORE_RESET: u64 = 5000;
/// Gas refund for SSTORE when clearing a storage slot (setting to zero).
pub const REFUND_SSTORE_CLEARS: i64 = 15000;

/// The standard cost of calldata token.
pub const STANDARD_TOKEN_COST: u64 = 4;
/// The cost of a non-zero byte in calldata.
pub const NON_ZERO_BYTE_DATA_COST: u64 = 68;
/// The multiplier for a non zero byte in calldata.
pub const NON_ZERO_BYTE_MULTIPLIER: u64 = NON_ZERO_BYTE_DATA_COST / STANDARD_TOKEN_COST;
/// The cost of a non-zero byte in calldata adjusted by [EIP-2028](https://eips.ethereum.org/EIPS/eip-2028).
pub const NON_ZERO_BYTE_DATA_COST_ISTANBUL: u64 = 16;
/// The multiplier for a non zero byte in calldata adjusted by [EIP-2028](https://eips.ethereum.org/EIPS/eip-2028).
pub const NON_ZERO_BYTE_MULTIPLIER_ISTANBUL: u64 =
    NON_ZERO_BYTE_DATA_COST_ISTANBUL / STANDARD_TOKEN_COST;
/// The cost floor per token as defined by [EIP-7623](https://eips.ethereum.org/EIPS/eip-7623).
pub const TOTAL_COST_FLOOR_PER_TOKEN: u64 = 10;

/// Gas cost for EOF CREATE instruction.
pub const EOF_CREATE_GAS: u64 = 32000;

// Berlin EIP-2929/EIP-2930 constants
/// Gas cost for accessing an address in the access list (EIP-2930).
pub const ACCESS_LIST_ADDRESS: u64 = 2400;
/// Gas cost for accessing a storage key in the access list (EIP-2930).
pub const ACCESS_LIST_STORAGE_KEY: u64 = 1900;

/// Gas cost for SLOAD when accessing a cold storage slot (EIP-2929).
pub const COLD_SLOAD_COST: u64 = 2100;
/// Gas cost for accessing a cold account (EIP-2929).
pub const COLD_ACCOUNT_ACCESS_COST: u64 = 2600;
/// Additional gas cost for accessing a cold account.
pub const COLD_ACCOUNT_ACCESS_COST_ADDITIONAL: u64 =
    COLD_ACCOUNT_ACCESS_COST - WARM_STORAGE_READ_COST;
/// Gas cost for reading from a warm storage slot (EIP-2929).
pub const WARM_STORAGE_READ_COST: u64 = 100;
/// Gas cost for SSTORE reset operation on a warm storage slot.
pub const WARM_SSTORE_RESET: u64 = SSTORE_RESET - COLD_SLOAD_COST;

/// EIP-3860 : Limit and meter initcode
pub const INITCODE_WORD_COST: u64 = 2;

/// Gas stipend provided to the recipient of a CALL with value transfer.
pub const CALL_STIPEND: u64 = 2300;

/// Init and floor gas from transaction
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitialAndFloorGas {
    /// Regular (non-state) portion of the initial intrinsic gas.
    ///
    /// Under EIP-8037, this is the part constrained by `TX_MAX_GAS_LIMIT`;
    /// state gas uses its own reservoir and is not subject to that cap.
    pub initial_regular_gas: u64,
    /// State gas charged at the intrinsic phase, before the first frame is
    /// entered.
    ///
    /// The state-dependent charges of the EIP-2780 runtime gas phase are not
    /// included here: they are recorded directly on the transaction-level gas.
    pub initial_state_gas: u64,
    /// If transaction is a Call and Prague is enabled
    /// floor_gas is at least amount of gas that is going to be spent.
    pub floor_gas: u64,
}

impl InitialAndFloorGas {
    /***** Constructors *****/

    /// Create a new InitialAndFloorGas instance.
    #[inline]
    pub const fn new(initial_regular_gas: u64, floor_gas: u64) -> Self {
        Self {
            initial_regular_gas,
            initial_state_gas: 0,
            floor_gas,
        }
    }

    /// Create a new InitialAndFloorGas instance with state gas tracking.
    #[inline]
    pub const fn new_with_state_gas(
        initial_regular_gas: u64,
        initial_state_gas: u64,
        floor_gas: u64,
    ) -> Self {
        Self {
            initial_regular_gas,
            initial_state_gas,
            floor_gas,
        }
    }

    /***** Simple getters *****/

    /// Regular (non-state) portion of the initial intrinsic gas.
    ///
    /// Under EIP-8037, this is the part constrained by `TX_MAX_GAS_LIMIT`;
    /// state gas uses its own reservoir and is not subject to that cap.
    #[inline]
    pub const fn initial_regular_gas(&self) -> u64 {
        self.initial_regular_gas
    }

    /// State gas charged before the first frame is entered.
    #[inline]
    pub const fn initial_state_gas_final(&self) -> u64 {
        self.initial_state_gas
    }

    /// EIP-7623 floor gas.
    #[inline]
    pub const fn floor_gas(&self) -> u64 {
        self.floor_gas
    }

    /// Total initial intrinsic gas: `initial_regular_gas + initial_state_gas`.
    #[inline]
    pub const fn initial_total_gas(&self) -> u64 {
        self.initial_regular_gas + self.initial_state_gas_final()
    }

    /***** Simple setters *****/

    /// Sets the `initial_regular_gas` field by mutable reference.
    #[inline]
    pub const fn set_initial_regular_gas(&mut self, initial_regular_gas: u64) {
        self.initial_regular_gas = initial_regular_gas;
    }

    /// Sets the `initial_state_gas` field by mutable reference.
    #[inline]
    pub const fn set_initial_state_gas(&mut self, initial_state_gas: u64) {
        self.initial_state_gas = initial_state_gas;
    }

    /// Sets the `floor_gas` field by mutable reference.
    #[inline]
    pub const fn set_floor_gas(&mut self, floor_gas: u64) {
        self.floor_gas = floor_gas;
    }

    /***** Builder with_* methods *****/

    /// Sets the `initial_regular_gas` field.
    #[inline]
    pub const fn with_initial_regular_gas(mut self, initial_regular_gas: u64) -> Self {
        self.initial_regular_gas = initial_regular_gas;
        self
    }

    /// Sets the `initial_state_gas` field.
    #[inline]
    pub const fn with_initial_state_gas(mut self, initial_state_gas: u64) -> Self {
        self.initial_state_gas = initial_state_gas;
        self
    }

    /// Sets the `floor_gas` field.
    #[inline]
    pub const fn with_floor_gas(mut self, floor_gas: u64) -> Self {
        self.floor_gas = floor_gas;
        self
    }

    /// Computes the regular gas budget and reservoir for the initial call frame.
    ///
    /// EIP-8037 reservoir model:
    ///   execution_gas = tx.gas_limit - intrinsic_gas  (= gas_limit parameter)
    ///   regular_gas_budget = min(execution_gas, TX_MAX_GAS_LIMIT - intrinsic_gas)
    ///   reservoir = execution_gas - regular_gas_budget
    ///
    /// Initial state gas is then deducted from the reservoir (spilling into the
    /// regular budget when the reservoir is insufficient).
    ///
    /// On mainnet (state gas disabled), reservoir = 0 and gas_limit is unchanged.
    ///
    /// All subtractions saturate at zero: callers normally guarantee
    /// `tx_gas_limit >= initial_total_gas` via validation, but if that invariant
    /// is violated the result clamps to `(0, 0)` instead of underflowing.
    ///
    /// Returns `(gas_limit, reservoir)`.
    pub fn initial_gas_and_reservoir(
        &self,
        tx_gas_limit: u64,
        tx_gas_limit_cap: u64,
    ) -> (u64, u64) {
        let execution_gas = tx_gas_limit.saturating_sub(self.initial_regular_gas());

        // System calls pass InitialAndFloorGas with all zeros and should not be
        // subject to the TX_MAX_GAS_LIMIT cap.
        let tx_gas_limit_cap = if self.initial_total_gas() == 0 {
            u64::MAX
        } else {
            tx_gas_limit_cap
        };

        let mut regular_gas_limit = core::cmp::min(tx_gas_limit, tx_gas_limit_cap)
            .saturating_sub(self.initial_regular_gas());
        let mut reservoir = execution_gas.saturating_sub(regular_gas_limit);

        // Deduct initial state gas from the reservoir. When the reservoir is
        // insufficient, the deficit is charged from the regular gas budget.
        if reservoir >= self.initial_state_gas {
            reservoir -= self.initial_state_gas;
        } else {
            regular_gas_limit =
                regular_gas_limit.saturating_sub(self.initial_state_gas - reservoir);
            reservoir = 0;
        }

        (regular_gas_limit, reservoir)
    }
}

/// Initial gas that is deducted for transaction to be included.
/// Initial gas contains initial stipend gas, gas for access list and input data.
///
/// # Returns
///
/// - Intrinsic gas
/// - Number of tokens in calldata
#[allow(clippy::too_many_arguments)]
pub fn calculate_initial_tx_gas(
    spec_id: SpecId,
    input: &[u8],
    is_create: bool,
    access_list_accounts: u64,
    access_list_storages: u64,
    authorization_list_num: u64,
    eip2780: Option<gas_params::Eip2780TxInfo>,
) -> InitialAndFloorGas {
    GasParams::new_spec(spec_id).initial_tx_gas(
        input,
        is_create,
        access_list_accounts,
        access_list_storages,
        authorization_list_num,
        eip2780,
    )
}

/// Initial gas that is deducted for transaction to be included.
/// Initial gas contains initial stipend gas, gas for access list and input data.
///
/// # Returns
///
/// - Intrinsic gas
/// - Number of tokens in calldata
pub fn calculate_initial_tx_gas_for_tx(
    tx: impl Transaction,
    spec: SpecId,
    eip2780: Option<gas_params::Eip2780TxInfo>,
) -> InitialAndFloorGas {
    GasParams::new_spec(spec).initial_tx_gas_for_tx(tx, eip2780)
}

/// Retrieve the total number of tokens in calldata.
#[inline]
pub fn get_tokens_in_calldata_istanbul(input: &[u8]) -> u64 {
    get_tokens_in_calldata(input, NON_ZERO_BYTE_MULTIPLIER_ISTANBUL)
}

/// Retrieve the total number of tokens in calldata.
#[inline]
pub fn get_tokens_in_calldata(input: &[u8], non_zero_data_multiplier: u64) -> u64 {
    let zero_data_len = input.iter().filter(|v| **v == 0).count() as u64;
    let non_zero_data_len = input.len() as u64 - zero_data_len;
    zero_data_len + non_zero_data_len * non_zero_data_multiplier
}
