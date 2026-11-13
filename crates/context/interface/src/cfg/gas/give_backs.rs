//! Give-backs to the reservoir alone, and the legacy share of a state charge.
//!
//! The switch that makes every give-back of state or history gas credit a [`GasTracker`]'s
//! reservoir and never its regular gas, the count of what regular gas paid for state and history
//! that a halt consumes, and the charge that takes a legacy share from regular gas before the
//! reservoir. The tracker's own documentation describes how each give-back is credited with the
//! switch on and off.

use super::GasTracker;

impl GasTracker {
    /// Whether every give-back of state or history gas credits the reservoir alone. Off by
    /// default, which is EIP-8037's tracker.
    ///
    /// With it on, no give-back credits regular gas, whichever pool paid the charge it gives back:
    /// a refill ([`refill_reservoir`](Self::refill_reservoir),
    /// [`refill_history`](Self::refill_history)), a failed frame's rollback
    /// ([`rollback_state_gas`](Self::rollback_state_gas)) and a reservoir a child returns
    /// ([`absorb_returned_reservoir`](Self::absorb_returned_reservoir)) land in the reservoir, so a
    /// frame's `remaining` rises within the frame only by the regular gas a child returns.
    /// [`state_gas_spilled`](Self::state_gas_spilled) counts what regular gas paid of state and
    /// history charges and no give-back lowers it. A halt consumes the frame's regular gas and
    /// the part of its charges regular gas paid, and gives it back the reservoir it received
    /// ([`consume_regular_paid_charges`](Self::consume_regular_paid_charges)).
    #[inline]
    pub const fn reservoir_only_give_backs(&self) -> bool {
        self.reservoir_only_give_backs
    }

    /// Sets whether every give-back of state or history gas credits the reservoir alone; see
    /// [`reservoir_only_give_backs`](Self::reservoir_only_give_backs). revm sets it on the
    /// transaction's gas and on every frame's from `Cfg::reservoir_only_give_backs`.
    #[inline]
    pub const fn set_reservoir_only_give_backs(&mut self, on: bool) {
        self.reservoir_only_give_backs = on;
    }

    /// Records a state gas cost that takes a legacy share from regular gas before it draws the
    /// reservoir.
    ///
    /// The share is `min(cost, legacy_share)`: it is drawn from `remaining` first, withheld part
    /// first ([`record_withheld_first_cost`](Self::record_withheld_first_cost)); the rest of the
    /// charge is then paid as [`record_state_cost`](Self::record_state_cost) pays it, from the
    /// reservoir and, past it, from `remaining` again. The share and that spill are one deduction
    /// from `remaining`, booked on [`state_gas_spilled`](Self::state_gas_spilled), and the whole
    /// charge is state gas.
    ///
    /// Returns `false` if `remaining` cannot pay the share and the spill together, whatever the
    /// reservoir holds, leaving the tracker's gas untouched and recording the regular gas left
    /// ([`failed_charge_remaining`](Self::failed_charge_remaining)). A share of zero is
    /// [`record_state_cost`](Self::record_state_cost).
    #[inline]
    #[must_use = "In case of not enough gas, the interpreter should halt with an out-of-gas error"]
    pub const fn record_state_cost_with_legacy_share(
        &mut self,
        cost: u64,
        legacy_share: u64,
    ) -> bool {
        let share = if legacy_share < cost {
            legacy_share
        } else {
            cost
        };
        let rest = cost - share;
        let from_reservoir = if self.reservoir < rest {
            self.reservoir
        } else {
            rest
        };
        let from_regular = cost - from_reservoir;
        if from_regular != 0 && !self.record_withheld_first_cost(from_regular) {
            return false;
        }
        self.reservoir -= from_reservoir;
        self.state_gas_spent = self
            .state_gas_spent
            .saturating_add(super::saturating_signed(cost));
        self.state_gas_spilled = self.state_gas_spilled.saturating_add(from_regular);
        true
    }

    /// Consumes, after [`rollback_state_gas`](Self::rollback_state_gas) of a frame that halts, the
    /// part of its state and history charges regular gas paid: the reservoir gives up
    /// `state_gas_spilled`, which is then zero, so it holds the reservoir the frame received, and
    /// what regular gas paid is consumed with the regular gas the halt spends.
    ///
    /// The rollback put every charge into the reservoir only with
    /// [`reservoir_only_give_backs`](Self::reservoir_only_give_backs) set. With it off the rollback
    /// credited the spill to `remaining` and zeroed it, so there is nothing left to take and this
    /// changes nothing.
    #[inline]
    pub const fn consume_regular_paid_charges(&mut self) {
        self.reservoir = self.reservoir.saturating_sub(self.state_gas_spilled);
        self.state_gas_spilled = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::GasTracker;

    /// A tracker with `remaining` regular gas and `reservoir`, its give-backs to the reservoir
    /// alone when `on`.
    fn tracker(remaining: u64, reservoir: u64, on: bool) -> GasTracker {
        let mut tracker = GasTracker::new(remaining, remaining, reservoir);
        tracker.set_reservoir_only_give_backs(on);
        tracker
    }

    /// `(remaining, reservoir, state_spent, history_spent, spilled)`.
    const fn figures(tracker: &GasTracker) -> (u64, u64, i64, i64, u64) {
        (
            tracker.remaining(),
            tracker.reservoir(),
            tracker.state_gas_spent(),
            tracker.history_gas_spent(),
            tracker.state_gas_spilled(),
        )
    }

    /// The switch is off on every tracker built, and is no part of a tracker that has it off.
    #[test]
    fn test_the_switch_is_off_by_default() {
        assert!(!GasTracker::new(1_000, 1_000, 0).reservoir_only_give_backs());
        assert!(!GasTracker::default().reservoir_only_give_backs());
        let mut on = GasTracker::default();
        on.set_reservoir_only_give_backs(true);
        assert!(on.reservoir_only_give_backs());
        assert_ne!(
            on,
            GasTracker::default(),
            "the switch is part of the tracker's value"
        );
    }

    /// A refill of state gas regular gas paid credits regular gas first with the switch off, and
    /// the reservoir alone with it on, the spill count standing.
    #[test]
    fn test_a_state_refill_credits_the_reservoir_alone_with_the_switch_on() {
        for on in [false, true] {
            let mut gas = tracker(1_000, 100, on);
            assert!(gas.record_state_cost(300));
            assert_eq!(figures(&gas), (800, 0, 300, 0, 200));
            gas.refill_reservoir(300);
            let expected = if on {
                (800, 300, 0, 0, 200)
            } else {
                (1_000, 100, 0, 0, 0)
            };
            assert_eq!(figures(&gas), expected, "switch on: {on}");
        }
    }

    /// A refill of history gas follows the same rule as a refill of state gas.
    #[test]
    fn test_a_history_refill_credits_the_reservoir_alone_with_the_switch_on() {
        for on in [false, true] {
            let mut gas = tracker(1_000, 100, on);
            assert!(gas.record_history_cost(300));
            gas.refill_history(300);
            let expected = if on {
                (800, 300, 0, 0, 200)
            } else {
                (1_000, 100, 0, 0, 0)
            };
            assert_eq!(figures(&gas), expected, "switch on: {on}");
        }
    }

    /// A reservoir a child returns unwinds the caller's spill with the switch off, and becomes the
    /// caller's reservoir as it is with it on.
    #[test]
    fn test_a_returned_reservoir_unwinds_nothing_with_the_switch_on() {
        for on in [false, true] {
            let mut gas = tracker(1_000, 0, on);
            assert!(gas.record_state_cost(300));
            gas.absorb_returned_reservoir(500);
            let expected = if on {
                (700, 500, 300, 0, 300)
            } else {
                (1_000, 200, 300, 0, 0)
            };
            assert_eq!(figures(&gas), expected, "switch on: {on}");
        }
    }

    /// With the switch on a revert puts every charge the frame made, net of what it was given
    /// back, into the reservoir, whichever pool paid it, and keeps the count of what regular gas
    /// paid; regular gas is left as it was. With it off the spill goes back to regular gas.
    #[test]
    fn test_a_rollback_puts_every_charge_into_the_reservoir_with_the_switch_on() {
        for on in [false, true] {
            let mut gas = tracker(1_000, 100, on);
            assert!(gas.record_state_cost(300));
            assert!(gas.record_history_cost(50));
            gas.refill_reservoir(20);
            gas.rollback_state_gas();
            let expected = if on {
                // 100 − 100 − 0 + 20 (refill) + 330 (net) = 350 of reservoir; 750 regular left.
                (750, 350, 0, 0, 250)
            } else {
                (1_000, 100, 0, 0, 0)
            };
            assert_eq!(figures(&gas), expected, "switch on: {on}");
        }
    }

    /// After a halting frame's rollback, consuming what regular gas paid leaves the reservoir the
    /// frame received with the switch on, and changes nothing with it off, where the rollback
    /// already credited it to regular gas.
    #[test]
    fn test_a_halt_gives_back_the_reservoir_the_frame_received() {
        for on in [false, true] {
            let mut gas = tracker(1_000, 100, on);
            assert!(gas.record_state_cost(300));
            assert!(gas.record_history_cost(50));
            gas.refill_reservoir(300);
            assert!(gas.record_state_cost_with_legacy_share(200, 80));
            gas.rollback_state_gas();
            gas.consume_regular_paid_charges();
            gas.spend_all();
            assert_eq!(gas.reservoir(), 100, "switch on: {on}");
            assert_eq!(gas.state_gas_spilled(), 0, "switch on: {on}");
            assert_eq!(gas.remaining(), 0, "switch on: {on}");
        }
    }

    /// A legacy share is paid from regular gas before the reservoir, which pays the rest; the
    /// regular part is counted as paid by regular gas, and the whole charge is state gas.
    #[test]
    fn test_a_legacy_share_comes_from_regular_gas_before_the_reservoir() {
        let mut gas = tracker(1_000, 500, true);
        assert!(gas.record_state_cost_with_legacy_share(300, 100));
        assert_eq!(figures(&gas), (900, 300, 300, 0, 100));

        // The share is capped at the charge.
        let mut gas = tracker(1_000, 500, true);
        assert!(gas.record_state_cost_with_legacy_share(60, 100));
        assert_eq!(figures(&gas), (940, 500, 60, 0, 60));

        // What the reservoir cannot pay after the share spills onto regular gas too.
        let mut gas = tracker(1_000, 150, true);
        assert!(gas.record_state_cost_with_legacy_share(300, 100));
        assert_eq!(figures(&gas), (850, 0, 300, 0, 150));
    }

    /// A share regular gas cannot pay is an out-of-gas whatever the reservoir holds, and leaves the
    /// tracker's gas untouched with the failure recorded; so is a share and a spill it cannot pay
    /// together.
    #[test]
    fn test_a_legacy_share_regular_gas_cannot_pay_runs_out_of_gas() {
        let mut gas = tracker(99, 1_000, true);
        let before = figures(&gas);
        assert!(!gas.record_state_cost_with_legacy_share(300, 100));
        assert_eq!(figures(&gas), before);
        assert_eq!(gas.failed_charge_remaining(), Some(99));

        let mut gas = tracker(149, 150, true);
        assert!(!gas.record_state_cost_with_legacy_share(300, 100));
        assert_eq!(gas.failed_charge_remaining(), Some(149));
    }

    /// The share draws the withheld part first, as a spill does.
    #[test]
    fn test_a_legacy_share_draws_the_withheld_part_first() {
        let mut gas = tracker(1_000, 500, true);
        gas.withhold(300);
        assert!(gas.record_state_cost_with_legacy_share(300, 100));
        assert_eq!(gas.withheld(), 200);
        assert_eq!(gas.spendable(), 700);
    }

    /// A share of zero is the charge EIP-8037 makes, with the switch on or off.
    #[test]
    fn test_a_legacy_share_of_zero_is_the_plain_state_charge() {
        for on in [false, true] {
            for (remaining, reservoir, cost) in
                [(1_000, 500, 300), (1_000, 100, 300), (200, 50, 300)]
            {
                let mut plain = tracker(remaining, reservoir, on);
                let mut shared = plain;
                assert_eq!(
                    plain.record_state_cost(cost),
                    shared.record_state_cost_with_legacy_share(cost, 0)
                );
                assert_eq!(plain, shared, "{remaining} {reservoir} {cost} {on}");
            }
        }
    }

    /// A give-back never lowers the count of what regular gas paid with the switch on, so a frame
    /// that paid a slot from regular gas and wrote it back still counts what it paid.
    #[test]
    fn test_no_give_back_lowers_the_count_of_what_regular_gas_paid() {
        let mut gas = tracker(1_000, 0, true);
        assert!(gas.record_state_cost_with_legacy_share(300, 100));
        assert!(gas.record_history_cost(40));
        gas.refill_reservoir(300);
        gas.refill_history(40);
        gas.absorb_returned_reservoir(500);
        assert_eq!(gas.state_gas_spilled(), 340);
        assert_eq!(gas.remaining(), 660);
    }
}
