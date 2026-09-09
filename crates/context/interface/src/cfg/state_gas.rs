//! Pricing hook for EIP-8037 state gas.
//!
//! Every state gas charge and every state gas refund is priced through
//! [`Host::state_gas_price`](crate::host::Host::state_gas_price), so a charge and the refund that
//! undoes it agree by construction. This module holds the value types that hook speaks in.

use super::gas_params::GasId;
use primitives::{Address, StorageKey};

/// What a state gas charge is attributed to: the account, plus the storage slot for slot-scoped
/// charges.
///
/// A pricing hook that only reads the [`GasId`] ignores this; one that prices the same charge
/// differently per location reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateGasSite {
    /// Account the charge is attributed to.
    pub address: Address,
    /// Storage slot, for charges that create or restore a single slot.
    pub slot: Option<StorageKey>,
}

impl StateGasSite {
    /// A charge attributed to a whole account.
    #[inline]
    pub const fn account(address: Address) -> Self {
        Self {
            address,
            slot: None,
        }
    }

    /// A charge attributed to one storage slot of an account.
    #[inline]
    pub const fn slot(address: Address, slot: StorageKey) -> Self {
        Self {
            address,
            slot: Some(slot),
        }
    }
}

/// A state gas charge that has been made and may still have to be undone.
///
/// Produced by the site that decides a refund is owed and consumed by the pricing hook, so the
/// decision stays independent of the price.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StateGasCharge {
    /// Which price to look up.
    pub id: GasId,
    /// Where the charge landed.
    pub site: StateGasSite,
    /// How many units of `id` were charged. One for the account-scoped charges, the code length
    /// for the per-byte ones.
    pub units: u64,
}

impl StateGasCharge {
    /// A single-unit charge.
    #[inline]
    pub const fn one(id: GasId, site: StateGasSite) -> Self {
        Self { id, site, units: 1 }
    }

    /// A charge of `units` units.
    #[inline]
    pub const fn units(id: GasId, site: StateGasSite, units: u64) -> Self {
        Self { id, site, units }
    }

    /// Total gas for this charge at `unit_price`.
    #[inline]
    pub const fn total(&self, unit_price: u64) -> u64 {
        unit_price.saturating_mul(self.units)
    }
}
