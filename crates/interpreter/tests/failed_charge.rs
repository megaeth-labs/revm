//! The record a failed regular charge leaves, at the instructions that charge a frame's gas.
//!
//! Each test runs a frame until it returns and reads its result's [`Gas`]: a frame whose regular
//! charge failed carries the regular gas it had left before that charge
//! ([`Gas::failed_charge_remaining`]), whatever the halt did to its gas, and a frame whose
//! charges all succeeded carries none.

use bytecode::{opcode::*, Bytecode};
use primitives::{hardfork::SpecId, Bytes};
use revm_interpreter::{
    host::DummyHost,
    instruction_table,
    instructions::gas_table_spec,
    interpreter::{EthInterpreter, ExtBytecode},
    Gas, InputsImpl, InstructionResult, Interpreter, InterpreterAction, SharedMemory,
};

const SPEC: SpecId = SpecId::OSAKA;

/// Runs `code` with `gas_limit`, of which `withheld` is withheld before the first instruction,
/// and returns the frame's result and gas.
fn run(code: &[u8], gas_limit: u64, withheld: u64) -> (InstructionResult, Gas) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl::default(),
        false,
        SPEC,
        gas_limit,
    );
    interpreter.gas.withhold(withheld);
    let mut host = DummyHost::new(SPEC);
    let table = instruction_table::<EthInterpreter, DummyHost>();
    let gas = gas_table_spec(SPEC);
    match interpreter.run_plain(&table, &gas, &mut host) {
        InterpreterAction::Return(result) => (result.result, result.gas),
        other => panic!("expected the frame to return, got {other:?}"),
    }
}

/// A frame whose charges all succeed carries no record, whether it stops, reverts or halts on
/// something other than a charge.
#[test]
fn test_a_frame_whose_charges_succeed_carries_no_record() {
    for (code, expected) in [
        (&[PUSH0, PUSH0, STOP][..], InstructionResult::Stop),
        (&[PUSH0, PUSH0, REVERT][..], InstructionResult::Revert),
        (&[PUSH0, INVALID][..], InstructionResult::InvalidFEOpcode),
    ] {
        let (result, gas) = run(code, 10_000, 0);

        assert_eq!(result, expected);
        assert_eq!(gas.failed_charge_remaining(), None, "{expected:?}");
    }
}

/// A static charge that fails halts `OutOfGas`, which zeroes the frame's gas; the record keeps
/// what the frame had before the charge.
#[test]
fn test_a_failed_static_charge_records_the_regular_gas_left() {
    // Two `PUSH0`s (4) leave 1 against the third's 2.
    let (result, gas) = run(&[PUSH0, PUSH0, PUSH0, STOP], 5, 0);

    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(gas.remaining(), 0, "the halt spent all");
    assert_eq!(gas.failed_charge_remaining(), Some(1));
    assert_eq!(gas.withheld_crossing(), None);
}

/// A frame with no gas left that tries a charge records zero, not nothing.
#[test]
fn test_a_failed_charge_with_nothing_left_records_zero() {
    let (result, gas) = run(&[PUSH0, PUSH0, STOP], 2, 0);

    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(gas.failed_charge_remaining(), Some(0));
}

/// A dynamic charge that fails after its instruction's static charge records what the frame had
/// before the dynamic charge.
#[test]
fn test_a_failed_dynamic_charge_records_the_regular_gas_left() {
    // `EXP(2, 0xff)`: the two pushes (6) and `EXP`'s static 10 leave 49 against the 50 its
    // one-byte exponent costs.
    let (result, gas) = run(&[PUSH1, 0xff, PUSH1, 2, EXP, STOP], 6 + 10 + 49, 0);

    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(gas.remaining(), 0, "the halt spent all");
    assert_eq!(gas.failed_charge_remaining(), Some(49));
}

/// A memory expansion that fails halts `MemoryOOG`, which zeroes nothing; the record holds the
/// regular gas left, which the frame still has.
#[test]
fn test_a_failed_memory_expansion_records_the_regular_gas_left() {
    // `PUSH0` (2) and `MLOAD` (3) leave 2 against the 3 one word of memory costs.
    let (result, gas) = run(&[PUSH0, MLOAD, STOP], 7, 0);

    assert_eq!(result, InstructionResult::MemoryOOG);
    assert_eq!(gas.failed_charge_remaining(), Some(2));
    assert_eq!(gas.remaining(), 2);
}

/// A regular charge the withheld part would have paid records the regular gas left, both parts,
/// and the crossing holds the same.
#[test]
fn test_a_crossing_records_the_regular_gas_left() {
    // 3 spendable and 97 withheld: the first `PUSH0` leaves 1 spendable against the second's 2.
    let (result, gas) = run(&[PUSH0, PUSH0, STOP], 100, 97);

    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(gas.remaining(), 0, "the halt spent all");
    assert_eq!(gas.failed_charge_remaining(), Some(1 + 97));
    assert_eq!(
        gas.withheld_crossing().map(|crossing| crossing.remaining()),
        Some(1 + 97)
    );
}

/// A charge beyond the whole regular gas of a frame that withholds part of it records the regular
/// gas left, both parts, and no crossing.
#[test]
fn test_a_failure_beyond_the_withheld_part_records_the_regular_gas_left() {
    // 16 spendable and 2 withheld: the pushes (5) and `EXP`'s static 10 leave 1 spendable against
    // the 50 its one-byte exponent costs, which the 3 left in all cannot pay either.
    let (result, gas) = run(&[PUSH1, 0xff, PUSH0, EXP, STOP], 3 + 2 + 10 + 3, 2);

    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(gas.failed_charge_remaining(), Some(1 + 2));
    assert_eq!(gas.withheld_crossing(), None);
}
