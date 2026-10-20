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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
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
    SelfDestruct {
        /// The account destructed.
        address: Address,
        /// The beneficiary.
        target: Address,
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
/// Built with [`FrameStart::new`], so a later field is not a breaking change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FrameStart {
    /// The kind of frame.
    pub kind: FrameStartKind,
    /// The frame's caller: the account the frame sees as `CALLER`.
    pub caller: Address,
    /// The account the frame runs as; `None` for a creation, whose address is decided later.
    pub target: Option<Address>,
    /// The value the start transfers; zero for `DELEGATECALL` and `STATICCALL`.
    pub value: U256,
    /// The regular gas the instruction took from the caller for the frame: the frame's gas limit
    /// without a value call's stipend, which the caller does not pay.
    pub forward: u64,
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
        }
    }
}
