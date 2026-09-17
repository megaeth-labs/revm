//! Fork-owned types of the EIP-7702 authorization phase.
//!
//! Re-exported from [`crate::pre_execution`]; kept apart so that the upstream file gains no new
//! type, only the pricing hook's call sites and the loop changes they need.

use context_interface::transaction::{AuthorizationTr, Transaction};
use primitives::{Address, U256};
use std::vec::Vec;

/// The scalar facts of one EIP-7702 authorization, lifted out of the transaction so the
/// authorization loop can hold the whole context instead of only the journal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Eip7702AuthFacts {
    /// Chain id the authorization is bound to; zero means any chain.
    pub chain_id: U256,
    /// Nonce the authority must be at for the authorization to apply.
    pub nonce: u64,
    /// Recovered authority, or `None` when signature recovery failed.
    pub authority: Option<Address>,
    /// Address to delegate to; zero clears the delegation.
    pub address: Address,
}

impl Eip7702AuthFacts {
    /// Lifts every authorization of `tx` into the scalar form.
    pub fn collect(tx: &impl Transaction) -> Vec<Self> {
        tx.authorization_list()
            .map(|authorization| Self {
                chain_id: authorization.chain_id(),
                nonce: authorization.nonce(),
                authority: authorization.authority(),
                address: authorization.address(),
            })
            .collect()
    }
}
