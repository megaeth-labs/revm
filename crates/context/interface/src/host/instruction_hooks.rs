//! Hooks an instruction calls on the [`Host`](super::Host) at a point inside it: once a state write
//! is complete ([`Host::write_completed`](super::Host::write_completed)), and once a frame's forward
//! is taken, before the frame's input is built ([`Host::frame_starting`](super::Host::frame_starting)).
//!
//! Both hooks get the running frame's [`GasTracker`](crate::cfg::gas::GasTracker), and may debit
//! and credit it; the instruction goes on with the gas as the hook left it. Their defaults touch
//! nothing and let the instruction go on.

use crate::context::{SStoreResult, SelfDestructResult};
use primitives::{Address, Bytes, StorageKey, U256};

/// A state write an instruction completed: the write is in the journal, and every charge the
/// instruction makes has been made.
///
/// Exhaustive on purpose: a new variant is a breaking change, so that every consumer handles every
/// kind of write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletedWrite<'a> {
    /// `SSTORE` wrote slot `key` of `address`.
    Sstore {
        /// The account whose storage was written.
        address: Address,
        /// The slot written.
        key: StorageKey,
        /// The slot's original, present and new values, as the journal reported them.
        result: &'a SStoreResult,
    },
    /// `LOG0`–`LOG4` appended a log for `address`.
    Log {
        /// The account that logged.
        address: Address,
        /// The number of topics, `0` to `4`.
        topics: u8,
        /// The length of the log's data, in bytes.
        data_len: u64,
    },
    /// `SELFDESTRUCT` destructed `address` in favour of `target`.
    ///
    /// The write changes no account's nonce or code, only balances: `address`'s info before it is
    /// its info after it with [`balance`](CompletedWrite::SelfDestruct::balance) as its balance,
    /// and `target`'s, when it is another account, its info after it less that balance. With the
    /// journal at the hook, that gives both accounts' change across the write.
    SelfDestruct {
        /// The account destructed.
        address: Address,
        /// The beneficiary.
        target: Address,
        /// The balance `address` held before the write. When `target` is another account, the
        /// write moved all of it to `target`. When `target` is `address`, the write burned it or
        /// left it, as the spec and EIP-6780 decide, and the account's balance after the write
        /// tells which.
        balance: U256,
        /// What the journal reported.
        result: &'a SelfDestructResult,
    },
}

/// What an instruction does once [`Host::write_completed`](super::Host::write_completed) has seen
/// its write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteVerdict {
    /// The instruction ends as it would without the hook.
    Continue,
    /// The instruction fails out of gas, as a failed charge does.
    OutOfGas,
    /// The frame ends as a `Revert` with this output, on its gas as the hook left it.
    Revert(Bytes),
}

/// The kind of frame an instruction is about to start.
///
/// Exhaustive on purpose: a new variant is a breaking change, so that every consumer handles every
/// kind of start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameStartKind {
    /// `CALL`.
    Call,
    /// `CALLCODE`.
    CallCode,
    /// `DELEGATECALL`.
    DelegateCall,
    /// `STATICCALL`.
    StaticCall,
    /// `CREATE`.
    Create,
    /// `CREATE2`.
    Create2,
}

/// A frame an instruction is about to start: every charge the instruction makes has been made, the
/// gas it forwards has been taken from the caller, and the frame's input is not built yet.
///
/// The hook is handed it mutably to name the child's [`reservoir`](Self::reservoir) if it takes
/// that itself; the instruction reads nothing else back.
///
/// Built with [`FrameStart::new`], so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FrameStart {
    /// The kind of frame.
    pub kind: FrameStartKind,
    /// The frame's caller: the account the frame sees as `CALLER`.
    pub caller: Address,
    /// The account the frame runs as. For a `CALL` or `STATICCALL`, the account called; for a
    /// `CALLCODE` or `DELEGATECALL`, the calling frame's own account, which runs the called
    /// account's code.
    ///
    /// For a `CREATE` or `CREATE2`, the address the creation would take, computed once: the
    /// handler builds the frame at the same address without computing it again. Whether the
    /// creation takes it is the outcome's to say (`CreateOutcome::address`): a creation that
    /// lands on an account that already has code or a nonce, one the handler refuses after the
    /// hook, and one an inspector or a consumer answers without a frame take none, though the
    /// start named the address. `None` only for a creation the handler fails before it takes an
    /// address, which reaches the hook only when EIP-8037 is off: an endowment above the caller's
    /// balance, a caller's nonce at its maximum, or a creating frame at the depth limit. With
    /// EIP-8037 on, the instruction ends such a creation before the hook.
    pub target: Option<Address>,
    /// The value the start transfers; zero for `DELEGATECALL` and `STATICCALL`.
    pub value: U256,
    /// The regular gas the instruction took from the caller for the frame: the frame's gas limit
    /// without a value call's stipend, which the caller does not pay.
    pub forward: u64,
    /// The child's reservoir, when the hook took it out of the caller's reservoir itself. `None`
    /// when the hook is called.
    ///
    /// `None` leaves the forward to the instruction, whose frame's input copies the caller's
    /// reservoir as the hook left it, and the caller keeps it, as upstream's instructions do.
    /// `Some` is the reservoir the frame's input carries in its place, and the caller keeps the
    /// reservoir the hook left it. The instruction does not check the amount against what the hook
    /// took.
    ///
    /// Either way the child's settlement replaces the caller's reservoir with the one the child
    /// returns. A hook that kept part of the caller's reservoir for it adds that back when the
    /// child settles ([`FrameSettle::held_reservoir`](super::FrameSettle::held_reservoir)).
    pub reservoir: Option<u64>,
}

impl FrameStart {
    /// A frame start.
    #[inline]
    pub const fn new(
        kind: FrameStartKind,
        caller: Address,
        target: Option<Address>,
        value: U256,
        forward: u64,
    ) -> Self {
        Self {
            kind,
            caller,
            target,
            value,
            forward,
            reservoir: None,
        }
    }
}
