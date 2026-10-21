//! The instructions' side of the [`Host`] instruction hooks: what an instruction does with the
//! answer of [`Host::write_completed`] and [`Host::frame_starting`].

use crate::{
    interpreter_types::{InterpreterTypes as ITy, LoopControl},
    Gas, Host, InstructionExecResult as Result, InstructionResult, Interpreter, InterpreterAction,
    StartCharge,
};
use context_interface::host::{CompletedWrite, FrameStart, WriteVerdict};
use primitives::{hints_util::cold_path, Bytes};

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
/// [`Host::frame_starting`], and takes the start charge the hook declared off `gas`, whatever
/// the answer, so the carrier is empty once the hook has answered. `true` returns the charge for
/// the frame's input; `false` fails the instruction out of gas, and the charge goes with the
/// start it was declared for.
#[inline]
pub(crate) fn frame_starting<H: Host + ?Sized>(
    gas: &mut Gas,
    host: &mut H,
    start: &FrameStart,
) -> core::result::Result<StartCharge, InstructionResult> {
    let admitted = host.frame_starting(start, gas.tracker_mut());
    let charged = gas.take_start_charge();
    if admitted {
        Ok(charged)
    } else {
        cold_path();
        Err(InstructionResult::OutOfGas)
    }
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
