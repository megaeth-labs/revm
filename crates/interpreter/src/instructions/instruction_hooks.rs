//! The instructions' side of the [`Host`] instruction hooks: what an instruction does with the
//! answer of [`Host::write_completed`] and [`Host::frame_starting`].

use crate::{
    interpreter_types::{InterpreterTypes as ITy, LoopControl},
    ChildShares, CreateInputs, Gas, Host, InstructionExecResult as Result, InstructionResult,
    Interpreter, InterpreterAction, StartCharge,
};
use context_interface::host::{CompletedWrite, FrameStart, WriteVerdict};
use primitives::{constants::CALL_STACK_LIMIT, hints_util::cold_path, Address, Bytes, U256};

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
/// a creating frame (at `depth`) at the depth limit. The handler builds the frame from the cache,
/// so the address and a `CREATE2`'s init-code hash are computed once.
#[inline]
pub(crate) fn cache_created_address<H: Host + ?Sized>(
    host: &mut H,
    inputs: &CreateInputs,
    depth: usize,
) -> Result {
    let (balance, nonce) = running_account(host, inputs.caller())?;
    if balance >= inputs.value() && nonce != u64::MAX && depth < CALL_STACK_LIMIT as usize {
        inputs.created_address(nonce);
    }
    Ok(())
}

/// The balance and nonce of `address`, the running frame's own account, for a hook's facts. The
/// journal holds that account warm, so reading it loads nothing cold.
#[inline]
pub(crate) fn running_account<H: Host + ?Sized>(
    host: &mut H,
    address: Address,
) -> core::result::Result<(U256, u64), InstructionResult> {
    let account = host
        .load_account_info_skip_cold_load(address, false, false)?
        .account;
    Ok((account.balance, account.nonce))
}

/// Takes the child's shares of the caller's reservoir and pools out of `gas`, by the schedule's
/// `reservoir_forward_percent` and `pool_forward_percent`s, and returns them: what the child's
/// tracker starts with ([`GasTracker::forward_to_child`](crate::GasTracker::forward_to_child)).
/// Called once the frame-start hook has answered, so the shares are of the reservoir and the pools
/// as the hook left them.
#[inline]
pub(crate) fn forward_to_child<H: Host + ?Sized>(gas: &mut Gas, host: &H) -> ChildShares {
    gas.tracker_mut().forward_to_child(host.gas_params())
}
