//! The start lifecycle: revm's pairing of every frame start with its settlement.
//!
//! A start opens where revm consumes a frame's init, just before it initialises the frame: a
//! frame's `NewFrame` action, consumed by [`EvmTr::frame_run`] (and by the inspector's
//! `inspect_frame_run`), opens a child start under the frame's own ([`open_frame_start`]); the
//! transaction's frame opens the transaction's start in [`Handler::execution`] (and the
//! inspector's `inspect_execution`), and so does the stand-in [`Handler::runtime_oog_result`]
//! builds for a frame that never runs ([`open_transaction_start`]).
//!
//! A start closes where revm settles the frame's result into its caller: a child's in
//! [`EvmTr::frame_return_result`], after `EthFrame::return_result` ([`close_frame_start`]), and
//! the transaction's in [`Handler::last_frame_result`], after
//! [`settle_first_frame`](crate::execution::settle_first_frame) ([`close_transaction_start`]).
//!
//! The lifecycle ends at one of the transaction's two boundaries: the successful one in
//! [`Handler::execution_result`], where every start must have settled ([`finish_transaction`]),
//! and the fatal one in [`Handler::catch_error`], which discards the starts still open
//! ([`discard_transaction`]).
//!
//! The record is revm's frame stack ([`FrameStack`]), which no inspector is handed. Each step is
//! handed to the context's lifecycle hooks ([`Host::frame_split`], [`Host::frame_settle`],
//! [`Host::frame_lifecycle_end`]). A step the record refuses ([`StartLifecycleError`]) is a fatal
//! error of the transaction in every build.
//!
//! An inspector's end callbacks may put another gas tracker or another outcome in a result's
//! place. revm marks the result for its start before them ([`mark_for_end_callbacks`]) and checks
//! the marks after them ([`check_end_callbacks`]), and an inspector's answer to a start is
//! recorded on it too ([`PendingStart::set_inspector_answer`]): the settlement reports all three
//! ([`FrameSettle`](context_interface::host::FrameSettle)).
//!
//! A chain's handler that overrides one of these methods makes the call the mainnet method makes,
//! in the same place: [`close_transaction_start`] in `last_frame_result`, [`finish_transaction`]
//! in `execution_result` and [`discard_transaction`] in `catch_error`. An EVM that reimplements
//! `frame_run` or `inspect_frame_run` calls [`open_frame_start`] for the init it consumed, and one
//! that reimplements `frame_return_result` calls [`close_frame_start`]. A runner that drives the
//! transaction's frame itself, outside [`Handler::execution`], opens the transaction's start first
//! ([`open_frame_start`] with the frame's init, or [`open_transaction_start`] for a frame it never
//! initialises).
//!
//! # Reuse after an error
//!
//! [`Handler::run`] hands an error to [`Handler::catch_error`], whose cleanup discards the
//! lifecycle, the journal's transaction, the local context and the frame stack. A caller of
//! [`Handler::run_without_catch_error`] that drops an error must make the same cleanup before it
//! reuses the EVM: call `catch_error`, or call [`discard_transaction`], discard the journal's
//! transaction, clear the local context and clear the frame stack ([`FrameStack::clear`]).
//! Discarding the lifecycle alone is not enough: a frame left on the stack makes the next
//! transaction's start a child of a frame with no start, which the record refuses.
//!
//! [`Handler::execution`]: crate::Handler::execution
//! [`Handler::runtime_oog_result`]: crate::Handler::runtime_oog_result
//! [`Handler::last_frame_result`]: crate::Handler::last_frame_result
//! [`Handler::execution_result`]: crate::Handler::execution_result
//! [`Handler::catch_error`]: crate::Handler::catch_error
//! [`Handler::run`]: crate::Handler::run
//! [`Handler::run_without_catch_error`]: crate::Handler::run_without_catch_error

use crate::{evm::ContextDbError, EthFrame, EvmTr, FrameResult};
use context_interface::{
    context::ContextError, host::SettleStatus, FrameStack, Host, PendingStart, StartLifecycleError,
};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, GasTracker, OutcomeMark, StartMark,
};
use std::string::ToString;

/// A refused step as the transaction's fatal error.
#[cold]
fn fatal<E>(error: StartLifecycleError) -> ContextError<E> {
    ContextError::Custom(error.to_string())
}

/// What a settlement reports of `result`'s status.
#[inline]
pub const fn settle_status(result: &FrameResult) -> SettleStatus {
    let result = result.instruction_result();
    if result.is_ok() {
        SettleStatus::Ok
    } else if result.is_revert() {
        SettleStatus::Revert
    } else {
        SettleStatus::Halt
    }
}

/// For a creation's `result`, whether it took an address; `None` for a call's.
#[inline]
const fn created(result: &FrameResult) -> Option<bool> {
    match result {
        FrameResult::Call(_) => None,
        FrameResult::Create(outcome) => Some(outcome.address.is_some()),
    }
}

/// Opens the start of the frame `init` makes, which revm consumed and initialises next: the child
/// of the frame on top of the stack, or the transaction's start when the stack is empty. Hands it
/// to [`Host::frame_split`].
///
/// # Errors
///
/// The record's refusal ([`FrameStack::open_start`]), as a fatal error.
#[inline]
pub fn open_frame_start<EVM: EvmTr + ?Sized>(
    evm: &mut EVM,
    init: &FrameInit,
) -> Result<(), ContextDbError<EVM::Context>> {
    let (ctx, _, _, frame_stack) = evm.all_mut();
    let split = frame_stack.open_start(init.depth).map_err(fatal)?;
    ctx.frame_split(&split);
    Ok(())
}

/// Opens the transaction's start for a transaction frame revm does not initialise: the stand-in
/// for a frame that never runs, or the frame of a runner that settles one it built itself. Hands
/// it to [`Host::frame_split`]. The frame stack must be empty.
///
/// # Errors
///
/// The record's refusal ([`FrameStack::open_start`]), as a fatal error.
#[inline]
pub fn open_transaction_start<EVM: EvmTr + ?Sized>(
    evm: &mut EVM,
) -> Result<(), ContextDbError<EVM::Context>> {
    let (ctx, _, _, frame_stack) = evm.all_mut();
    debug_assert!(
        frame_stack.index().is_none(),
        "the transaction's start opens on an empty frame stack"
    );
    let split = frame_stack.open_start(0).map_err(fatal)?;
    ctx.frame_split(&split);
    Ok(())
}

/// A frame's result on its way into its caller: what its settlement reports, read before
/// `EthFrame::return_result` consumes it ([`close_frame_start`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Returning {
    depth: usize,
    status: SettleStatus,
    created: Option<bool>,
    held_reservoir: u64,
}

impl Returning {
    /// `result`, about to be returned into `caller`, the frame on top of the stack.
    #[inline]
    pub const fn new(result: &FrameResult, caller: &EthFrame<EthInterpreter>) -> Self {
        Self {
            depth: caller.depth + 1,
            status: settle_status(result),
            created: created(result),
            held_reservoir: caller.interpreter.gas.reservoir(),
        }
    }
}

/// Closes the start of the frame whose result was just returned into the frame on top of
/// `frame_stack`, its caller, after `EthFrame::return_result` settled it, and hands the settlement
/// to [`Host::frame_settle`] with the caller's gas. `caller` reaches the caller's `EthFrame` from
/// the stack's frame.
///
/// # Errors
///
/// [`StartLifecycleError::NotOpen`] when the caller has no open start, as a fatal error.
#[inline]
pub fn close_frame_start<CTX, F, E>(
    ctx: &mut CTX,
    frame_stack: &mut FrameStack<F>,
    returning: Returning,
    caller: impl FnOnce(&mut F) -> &mut EthFrame<EthInterpreter>,
) -> Result<(), ContextError<E>>
where
    CTX: Host + ?Sized,
{
    let settle = frame_stack
        .close_start(
            returning.depth,
            returning.status,
            returning.created,
            returning.held_reservoir,
        )
        .map_err(fatal)?;
    let caller = caller(frame_stack.get());
    ctx.frame_settle(&settle, caller.interpreter.gas.tracker_mut());
    Ok(())
}

/// Closes the transaction's start into the transaction-level `gas`, after
/// [`settle_first_frame`](crate::execution::settle_first_frame) settled `result` into it, and
/// hands the settlement to [`Host::frame_settle`]. `held_reservoir` is the reservoir `gas` held
/// before that settlement. After a halt, revm's halt rule follows the hook: the regular gas left
/// in `gas` is consumed, as the settlement's own refunds are. The frame stack must be empty.
///
/// # Errors
///
/// [`StartLifecycleError::NotOpen`] when the transaction's start is not open, as a fatal error.
#[inline]
pub fn close_transaction_start<EVM: EvmTr + ?Sized>(
    evm: &mut EVM,
    result: &FrameResult,
    gas: &mut GasTracker,
    held_reservoir: u64,
) -> Result<(), ContextDbError<EVM::Context>> {
    let status = settle_status(result);
    let (ctx, _, _, frame_stack) = evm.all_mut();
    debug_assert!(
        frame_stack.index().is_none(),
        "the transaction's start closes on an empty frame stack"
    );
    let settle = frame_stack
        .close_start(0, status, created(result), held_reservoir)
        .map_err(fatal)?;
    ctx.frame_settle(&settle, gas);
    if status == SettleStatus::Halt {
        gas.spend_all();
    }
    Ok(())
}

/// The successful boundary: every start the transaction opened must have settled. Ends the
/// lifecycle ([`LifecycleEnd::Settled`](context_interface::host::LifecycleEnd::Settled), handed to
/// [`Host::frame_lifecycle_end`] when a lifecycle was under way) and resets the record for the
/// next transaction.
///
/// # Errors
///
/// [`StartLifecycleError::Unsettled`] when a start is still open, a missed settlement, as a fatal
/// error. The lifecycle is then discarded
/// ([`LifecycleEnd::Discarded`](context_interface::host::LifecycleEnd::Discarded)), so the handler's
/// `catch_error` that reports the error owes no second end.
#[inline]
pub fn finish_transaction<EVM: EvmTr + ?Sized>(
    evm: &mut EVM,
) -> Result<(), ContextDbError<EVM::Context>> {
    let (ctx, _, _, frame_stack) = evm.all_mut();
    match frame_stack.settle_lifecycle() {
        Ok(end) => {
            if let Some(end) = end {
                ctx.frame_lifecycle_end(end);
            }
            Ok(())
        }
        Err(error) => {
            if let Some(end) = frame_stack.discard_lifecycle() {
                ctx.frame_lifecycle_end(end);
            }
            Err(fatal(error))
        }
    }
}

/// The fatal boundary: discards the starts still open, ends the lifecycle
/// ([`LifecycleEnd::Discarded`](context_interface::host::LifecycleEnd::Discarded), handed to
/// [`Host::frame_lifecycle_end`] when a lifecycle was under way) and resets the record for the
/// next transaction. Idempotent: a lifecycle already ended owes nothing.
///
/// It resets the lifecycle only. Before the EVM runs another transaction, the frame stack must be
/// cleared too, as [`Handler::catch_error`](crate::Handler::catch_error) does (see the module's
/// "Reuse after an error").
#[inline]
pub fn discard_transaction<EVM: EvmTr + ?Sized>(evm: &mut EVM) {
    let (ctx, _, _, frame_stack) = evm.all_mut();
    if let Some(end) = frame_stack.discard_lifecycle() {
        ctx.frame_lifecycle_end(end);
    }
}

/// Marks `result`, the result of `start`, for the start, just before an inspector's end
/// callbacks (`frame_end`, `call_end`, `create_end`): its gas tracker with a [`StartMark`] and its
/// outcome with an [`OutcomeMark`]. Neither mark is part of the values' public semantics.
#[inline]
pub const fn mark_for_end_callbacks(result: &mut FrameResult, start: &PendingStart) {
    let id = start.id();
    result
        .gas_mut()
        .tracker_mut()
        .set_start_mark(StartMark::for_start(id));
    let mark = OutcomeMark::for_start(id);
    match result {
        FrameResult::Call(outcome) => outcome.start_mark = mark,
        FrameResult::Create(outcome) => outcome.start_mark = mark,
    }
}

/// Records on `start`, just after an inspector's end callbacks, what they put in place of
/// `result`'s gas tracker and outcome: a tracker without the start's mark
/// ([`PendingStart::set_gas_replaced`]) and an outcome without it
/// ([`PendingStart::set_outcome_rebuilt`]). A callback that edits the values in place, or copies
/// them, keeps the marks.
#[inline]
pub const fn check_end_callbacks(result: &FrameResult, start: &mut PendingStart) {
    let id = start.id();
    if !result.gas().tracker().start_mark().is_for(id) {
        start.set_gas_replaced();
    }
    let outcome_mark = match result {
        FrameResult::Call(outcome) => outcome.start_mark,
        FrameResult::Create(outcome) => outcome.start_mark,
    };
    if !outcome_mark.is_for(id) {
        start.set_outcome_rebuilt();
    }
}
