//! Identifies the MegaETH fork of revm.
//!
//! Downstream crates assert against [`FORK_TAG`](crate::megaeth::FORK_TAG) at compile time so that a
//! workspace which resolves `revm` from crates.io instead of the fork fails to
//! build with a clear message.

/// Git tag of the most recent fork release at or before this commit.
///
/// Only the tagged commit itself is a release; an untagged commit on `main`
/// still carries the previous value.  It is bumped in the release commit of
/// every `v<upstream>-mega.N` tag.
pub const FORK_TAG: &str = "v40.0.3-mega.1";
