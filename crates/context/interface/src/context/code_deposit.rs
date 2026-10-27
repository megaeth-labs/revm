//! The code deposit `return_create` is about to commit, as
//! [`ContextTr::admit_code_deposit`](super::ContextTr::admit_code_deposit) sees it.

use crate::cfg::gas::GasTracker;
use primitives::{Address, Bytes};

/// A code deposit `return_create` has decided to make.
///
/// The creation returned successfully, its code passed every check `return_create` makes (the
/// code-size limit, EIP-3541), and every charge for the deposit is recorded on the creating
/// frame's gas: the code-deposit cost, and under EIP-8037 the cost of hashing the code, its state
/// gas and its history gas. What is left is the journal checkpoint commit and the code write.
/// [`ContextTr::admit_code_deposit`](super::ContextTr::admit_code_deposit) is asked to admit it
/// at exactly that point. Before Homestead that includes a deposit the frame could not pay, which
/// `return_create` makes with empty code and no charge instead of failing the creation.
///
/// The charges are read off the two trackers: [`gas_before`](Self::gas_before), the creating
/// frame's gas before the first deposit charge, and [`gas_after`](Self::gas_after), the same gas
/// with every deposit charge recorded. Nothing else touches the frame's gas between the two.
///
/// The fields are public to read. The type is `#[non_exhaustive]`, so it is built with
/// [`new`](Self::new) outside this crate, and a field added later is not a breaking change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CodeDeposit<'a> {
    /// The address the code is deployed at.
    pub address: Address,
    /// The code to be deployed. Empty when a pre-Homestead creation could not pay the
    /// code-deposit cost, which deploys empty code instead of failing.
    pub code: &'a Bytes,
    /// The creating frame's gas before the deposit's charges. A refused deposit puts the frame's
    /// gas back to this.
    pub gas_before: &'a GasTracker,
    /// The creating frame's gas with every deposit charge recorded: what the frame returns with
    /// when the deposit is admitted.
    ///
    /// Before Homestead, when the frame could not pay the code-deposit cost, nothing was charged
    /// and [`code`](Self::code) is empty. If that failed charge was a crossing of the frame's
    /// withheld gas, this carries its record
    /// ([`GasTracker::withheld_crossing`](crate::cfg::gas::GasTracker::withheld_crossing)), which
    /// [`gas_before`](Self::gas_before) lacks: the record stays on the frame if the deposit is
    /// admitted, and a refusal discards it.
    pub gas_after: &'a GasTracker,
}

impl<'a> CodeDeposit<'a> {
    /// Creates the deposit of `code` at `address`, with the creating frame's gas before and after
    /// the deposit's charges.
    #[inline]
    pub const fn new(
        address: Address,
        code: &'a Bytes,
        gas_before: &'a GasTracker,
        gas_after: &'a GasTracker,
    ) -> Self {
        Self {
            address,
            code,
            gas_before,
            gas_after,
        }
    }

    /// The regular gas the deposit charged: the code-deposit cost and, under EIP-8037, the cost
    /// of hashing the code. State and history gas that spilled onto regular gas are not counted
    /// here; they are part of [`state_gas`](Self::state_gas) and
    /// [`history_gas`](Self::history_gas).
    #[inline]
    pub const fn regular_gas(&self) -> u64 {
        let drawn = self
            .gas_before
            .remaining()
            .saturating_sub(self.gas_after.remaining());
        let spilled = self
            .gas_after
            .state_gas_spilled()
            .saturating_sub(self.gas_before.state_gas_spilled());
        drawn.saturating_sub(spilled)
    }

    /// The EIP-8037 state gas the deposit charged for the code, priced through
    /// [`Host::state_gas_price`](crate::Host::state_gas_price). Zero when the schedule does not
    /// price deposited code.
    ///
    /// The price was looked up before the deposit is offered, so the lookup is made whether or
    /// not the deposit is admitted.
    #[inline]
    pub const fn state_gas(&self) -> u64 {
        charged(
            self.gas_before.state_gas_spent(),
            self.gas_after.state_gas_spent(),
        )
    }

    /// The history gas the deposit charged for the code. Zero when the schedule does not price
    /// history bytes.
    #[inline]
    pub const fn history_gas(&self) -> u64 {
        charged(
            self.gas_before.history_gas_spent(),
            self.gas_after.history_gas_spent(),
        )
    }
}

/// What a signed net counter went up by from `before` to `after`, as an amount charged.
const fn charged(before: i64, after: i64) -> u64 {
    let delta = after.saturating_sub(before);
    if delta > 0 {
        delta as u64
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::{CodeDeposit, GasTracker};
    use primitives::{Address, Bytes};

    /// The charges come back apart from the two trackers, a state and a history charge that
    /// spilled onto regular gas included: what spilled is counted as state and history gas, not
    /// as regular gas.
    #[test]
    fn test_the_charges_are_read_off_the_two_trackers() {
        let mut before = GasTracker::new(100_000, 100_000, 1_000);
        assert!(before.record_state_cost(300));
        let mut after = before;
        assert!(after.record_regular_cost(6_400));
        assert!(after.record_regular_cost(6));
        // 700 of the reservoir left: the state charge spills 1_300, the history charge all 500.
        assert!(after.record_state_cost(2_000));
        assert!(after.record_history_cost(500));
        let code = Bytes::from_static(&[0; 32]);

        let deposit = CodeDeposit::new(Address::ZERO, &code, &before, &after);

        assert_eq!(deposit.regular_gas(), 6_406);
        assert_eq!(deposit.state_gas(), 2_000);
        assert_eq!(deposit.history_gas(), 500);
        assert_eq!(
            before.remaining() - after.remaining(),
            6_406 + 1_300 + 500,
            "regular gas paid the regular charges and both spills"
        );
    }

    /// A deposit that charged nothing reads as nothing, whatever the frame charged before it.
    #[test]
    fn test_a_deposit_that_charged_nothing() {
        let mut before = GasTracker::new(1_000, 1_000, 0);
        before.refill_reservoir(50);
        assert!(before.record_regular_cost(10));
        let code = Bytes::new();

        let deposit = CodeDeposit::new(Address::ZERO, &code, &before, &before);

        assert_eq!(
            (
                deposit.regular_gas(),
                deposit.state_gas(),
                deposit.history_gas()
            ),
            (0, 0, 0)
        );
    }
}
