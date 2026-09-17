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
    /// Lifts the authorizations of `tx` that can apply on `chain_id` into the scalar form.
    ///
    /// An authorization bound to another chain, or with nonce `u64::MAX`, is dropped before its
    /// authority is recovered, at the point where the authorization loop would skip it, so its
    /// signature is never recovered.
    ///
    /// The list is buffered rather than streamed because the loop hands the whole context to the
    /// pricing hook. Each entry is 88 bytes, and every authorization has already paid at least
    /// the per-authorization intrinsic gas (7,816 under EIP-2780), so the buffer is proportional
    /// to gas the sender paid.
    pub fn collect(tx: &impl Transaction, chain_id: u64) -> Vec<Self> {
        let chain_id = U256::from(chain_id);
        tx.authorization_list()
            .filter_map(|authorization| {
                let auth_chain_id = authorization.chain_id();
                let nonce = authorization.nonce();
                let wrong_chain = !auth_chain_id.is_zero() && auth_chain_id != chain_id;
                if wrong_chain || nonce == u64::MAX {
                    return None;
                }
                Some(Self {
                    chain_id: auth_chain_id,
                    nonce,
                    authority: authorization.authority(),
                    address: authorization.address(),
                })
            })
            .collect()
    }
}
