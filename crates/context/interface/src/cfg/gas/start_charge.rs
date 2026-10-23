//! The start charge a [`Host::frame_starting`](crate::Host::frame_starting) hook declares on the
//! [`FrameStart`](crate::host::FrameStart) it is handed, and the instruction copies onto the
//! frame's input.
//!
//! The hook charges the caller's gas for the frame it starts; the part of that charge the frame's
//! failure gives back is the start charge. The hook declares it on the start
//! ([`FrameStart::charge`](crate::host::FrameStart::charge)), the instruction copies it onto the
//! frame's input when the hook admits the start, and the handler refunds it to the caller when the
//! frame fails ([`GasTracker::refill_start_charge`]).

use super::{GasTracker, PoolAmounts, PoolId};

/// What a frame start charged its caller and the frame's failure gives back: history gas and an
/// amount of each pool.
///
/// Built with [`StartCharge::new`], so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct StartCharge {
    /// History gas the start charged, refilled with
    /// [`refill_history`](GasTracker::refill_history) when the frame fails.
    pub history: u64,
    /// What the start drew from each of the caller's pools, refilled with
    /// [`refill_pool`](GasTracker::refill_pool) when the frame fails.
    pub pools: PoolAmounts,
}

impl StartCharge {
    /// A start charge of `history` gas and `pools` from the caller's pools.
    #[inline]
    pub const fn new(history: u64, pools: PoolAmounts) -> Self {
        Self { history, pools }
    }

    /// Nothing to refund.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.history == 0 && self.pools.is_zero()
    }
}

impl GasTracker {
    /// Gives `charge` back to this, the caller's, tracker: its history gas with
    /// [`refill_history`](Self::refill_history), and each pool's amount with
    /// [`refill_pool`](Self::refill_pool).
    ///
    /// The handler calls it when a frame whose start charged `charge` fails
    /// (`FrameResult::refundable_start_charge`); a consumer that settles a frame's result itself
    /// calls it on the same condition.
    #[inline]
    pub const fn refill_start_charge(&mut self, charge: StartCharge) {
        self.refill_history(charge.history);
        let mut index = 0;
        while index < PoolId::COUNT {
            let pool = PoolId::ALL[index];
            self.refill_pool(pool, charge.pools.get(pool));
            index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GasTracker, PoolAmounts, PoolId, StartCharge};

    /// The refill gives the history back the way `refill_history` does and each pool's amount to
    /// its own pool, so a caller that drew them is where it was before the start.
    #[test]
    fn test_a_refilled_start_charge_puts_the_caller_back() {
        let mut caller = GasTracker::new(10_000, 10_000, 0);
        for pool in PoolId::ALL {
            caller.set_pool_remaining(pool, 1_000);
        }
        let before = caller;
        let charge = StartCharge::new(
            3_520,
            PoolAmounts::ZERO
                .with(PoolId::new(0), 40)
                .with(PoolId::new(1), 1),
        );
        assert!(caller.record_history_cost(charge.history));
        for pool in PoolId::ALL {
            assert!(caller.record_pool(pool, charge.pools.get(pool)));
        }

        caller.refill_start_charge(charge);

        assert_eq!(caller, before);
        assert!(!charge.is_zero());
        assert!(StartCharge::default().is_zero());
        assert!(!StartCharge::new(0, PoolAmounts::ZERO.with(PoolId::new(2), 1)).is_zero());
    }
}
