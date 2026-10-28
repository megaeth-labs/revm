//! An action set on an interpreter before it runs is what the run returns, before any
//! instruction.
//!
//! A consumer sets a frame's next action between the frame's start and its run (or after a child
//! returned into it) with `LoopControl::set_action`. The plain run returns that action as it was
//! set: no instruction runs, so the frame's gas, program counter and stack are as the consumer
//! left them, and the interpreter is ready to run again.

use bytecode::{opcode::*, Bytecode};
use primitives::{hardfork::SpecId, Bytes};
use revm_interpreter::{
    host::DummyHost,
    instruction_table,
    instructions::gas_table_spec,
    interpreter::{EthInterpreter, ExtBytecode},
    interpreter_types::{Jumps, LoopControl},
    InputsImpl, InstructionResult, Interpreter, InterpreterAction, SharedMemory,
};

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 10_000;

/// Code that would push twice and stop: every instruction it runs moves the program counter, the
/// stack and the gas.
const CODE: &[u8] = &[PUSH0, PUSH0, STOP];

/// An interpreter on [`CODE`] with a revert set as its next action, and that action.
fn pending() -> (Interpreter<EthInterpreter>, InterpreterAction) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::from_static(CODE))),
        InputsImpl::default(),
        false,
        SPEC,
        GAS_LIMIT,
    );
    let action = InterpreterAction::new_return(
        InstructionResult::Revert,
        Bytes::from_static(b"set before the run"),
        interpreter.gas,
    );
    interpreter.bytecode.set_action(action.clone());
    (interpreter, action)
}

/// Asserts that `interpreter` ran no instruction and is ready to run again.
fn assert_ran_nothing(interpreter: &Interpreter<EthInterpreter>) {
    assert_eq!(
        interpreter.bytecode.pc(),
        0,
        "no instruction moved the counter"
    );
    assert_eq!(interpreter.stack.len(), 0, "no instruction pushed");
    assert_eq!(
        interpreter.gas.remaining(),
        GAS_LIMIT,
        "no instruction charged"
    );
    assert!(interpreter.bytecode.is_not_end(), "the action was taken");
}

#[test]
fn test_run_plain_returns_a_pending_action_before_any_instruction() {
    let (mut interpreter, action) = pending();
    let table = instruction_table::<EthInterpreter, DummyHost>();

    let returned = interpreter.run_plain(&table, &gas_table_spec(SPEC), &mut DummyHost::new(SPEC));

    assert_eq!(returned, action);
    assert_ran_nothing(&interpreter);
}

#[test]
fn test_run_plain_as_output_returns_a_pending_action_before_any_instruction() {
    let (mut interpreter, action) = pending();
    let table = instruction_table::<EthInterpreter, DummyHost>();

    let returned =
        interpreter.run_plain_as_output(&table, &gas_table_spec(SPEC), &mut DummyHost::new(SPEC));

    assert_eq!(returned, action);
    assert_ran_nothing(&interpreter);
}
