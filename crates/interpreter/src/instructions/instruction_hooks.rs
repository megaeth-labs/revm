//! The instructions' side of the [`Host`] instruction hooks: what an instruction does with the
//! answer of [`Host::write_completed`] and [`Host::frame_starting`].

use crate::{
    interpreter_types::{InterpreterTypes as ITy, LoopControl},
    CreateInputs, Gas, Host, InstructionExecResult as Result, InstructionResult, Interpreter,
    InterpreterAction, StartCharge,
};
use context_interface::host::{CompletedWrite, FrameStart, WriteVerdict};
use primitives::{constants::CALL_STACK_LIMIT, hints_util::cold_path, Bytes};

/// Hands `write`, which the running instruction completed after its last charge, to
/// [`Host::write_completed`] with the frame's gas, and applies the verdict: the instruction goes
/// on, fails out of gas, or ends the frame as a `Revert` with the verdict's output on the gas as
/// the hook left it.
#[inline]
pub(crate) fn write_completed<IT: ITy, H: Host + ?Sized>(
    interpreter: &mut Interpreter<IT>,
    host: &mut H,
    write: CompletedWrite<'_>,
) -> Result {
    match host.write_completed(write, interpreter.gas.tracker_mut()) {
        WriteVerdict::Continue => Ok(()),
        WriteVerdict::OutOfGas => {
            cold_path();
            Err(InstructionResult::OutOfGas)
        }
        WriteVerdict::Revert(output) => revert(interpreter, output),
    }
}

/// Ends the frame as a `Revert` with `output`, as `REVERT` does.
#[cold]
#[inline(never)]
fn revert<IT: ITy>(interpreter: &mut Interpreter<IT>, output: Bytes) -> Result {
    interpreter
        .bytecode
        .set_action(InterpreterAction::new_return(
            InstructionResult::Revert,
            output,
            interpreter.gas,
        ));
    Err(InstructionResult::Revert)
}

/// Hands `start`, whose forward the running instruction has taken from `gas`, to
/// [`Host::frame_starting`]. `true` returns the start charge the hook declared on `start`, for
/// the frame's input; `false` fails the instruction out of gas, and the charge goes with the
/// start it was declared for.
#[inline]
pub(crate) fn frame_starting<H: Host + ?Sized>(
    gas: &mut Gas,
    host: &mut H,
    mut start: FrameStart,
) -> core::result::Result<StartCharge, InstructionResult> {
    if host.frame_starting(&mut start, gas.tracker_mut()) {
        Ok(start.charge)
    } else {
        cold_path();
        Err(InstructionResult::OutOfGas)
    }
}

/// Computes the address the creation `inputs` takes into their cache, for the frame-start hook,
/// when EIP-8037 is off; with it on, the creation instruction's EIP-8037 branch computes it. It does
/// so after the checks that branch makes, so a creation the handler fails before it takes an
/// address gets none: an endowment above the caller's balance, a caller's nonce at its maximum, or
/// a creating frame at `depth` the depth limit. The handler builds the frame from the cache, so the
/// address and a `CREATE2`'s init-code hash are computed once. The caller is the running frame's
/// account, which the journal holds, so reading its nonce loads nothing cold.
#[inline]
pub(crate) fn cache_created_address<H: Host + ?Sized>(
    host: &mut H,
    inputs: &CreateInputs,
    depth: usize,
) -> Result {
    let caller = host
        .load_account_info_skip_cold_load(inputs.caller(), false, false)?
        .account;
    let (balance, nonce) = (caller.balance, caller.nonce);
    if balance >= inputs.value() && nonce != u64::MAX && depth < CALL_STACK_LIMIT as usize {
        inputs.created_address(nonce);
    }
    Ok(())
}

/// Takes the child's share of the caller's byte pool out of `gas`, by the schedule's
/// `bytes_forward_percent`, and returns it: the pool the child starts with. Called once the
/// frame-start hook has answered, so the share is of the pool as the hook left it.
#[inline]
pub(crate) fn forward_bytes<H: Host + ?Sized>(gas: &mut Gas, host: &H) -> u64 {
    let share = host.gas_params().bytes_forward_share(gas.bytes_remaining());
    gas.tracker_mut().forward_bytes(share);
    share
}
