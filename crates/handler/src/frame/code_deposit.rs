//! The admission point of a code deposit in [`return_create`](super::return_create).

use context_interface::{
    context::CodeDeposit, journaled_state::JournalCheckpoint, ContextTr, JournalTr,
};
use interpreter::{Gas, InstructionResult, InterpreterResult};
use primitives::Address;

/// Asks the context to admit the code deposit `return_create` is about to commit at `address`
/// ([`ContextTr::admit_code_deposit`]), and returns whether it did.
///
/// `interpreter_result` holds the code and the creating frame's gas with every deposit charge
/// recorded; `gas_before_deposit` is that gas before the first of them. A refused deposit reverts
/// `checkpoint`, puts the frame's gas back to `gas_before_deposit` and turns the result into a
/// `Revert` carrying the output the context named; `return_create` then returns without
/// committing.
///
/// Inlined into `return_create`, so under the default hook, which admits, it compiles away with
/// the copy of the frame's gas it reads.
#[inline(always)]
pub(super) fn admit_code_deposit<CTX: ContextTr>(
    context: &mut CTX,
    checkpoint: JournalCheckpoint,
    interpreter_result: &mut InterpreterResult,
    gas_before_deposit: &Gas,
    address: Address,
) -> bool {
    let admitted = context.admit_code_deposit(&CodeDeposit::new(
        address,
        &interpreter_result.output,
        gas_before_deposit.tracker(),
        interpreter_result.gas.tracker(),
    ));
    let Err(output) = admitted else {
        return true;
    };
    context.journal_mut().checkpoint_revert(checkpoint);
    interpreter_result.gas = *gas_before_deposit;
    interpreter_result.result = InstructionResult::Revert;
    interpreter_result.output = output;
    false
}
