//! The record a failed charge leaves, at the instructions that charge a frame's gas.
//!
//! Each test runs a frame until it returns and reads its result's [`Gas`]: a frame whose charge
//! failed carries the regular gas it had left before that charge
//! ([`Gas::failed_charge_remaining`]), whatever the halt did to its gas, and a frame whose
//! charges all succeeded carries none.

use bytecode::{opcode::*, Bytecode};
use context_interface::host::LoadError;
use primitives::{hardfork::SpecId, Bytes};
use revm_interpreter::{
    host::DummyHost,
    instruction_table,
    instructions::gas_table_spec,
    interpreter::{EthInterpreter, ExtBytecode},
    Gas, InputsImpl, Instruction, InstructionContext, InstructionExecResult, InstructionResult,
    Interpreter, InterpreterAction, SharedMemory,
};

const SPEC: SpecId = SpecId::OSAKA;

/// Runs `code` with `gas_limit`, of which `withheld` is withheld before the first instruction,
/// and returns the frame's result and gas.
fn run(code: &[u8], gas_limit: u64, withheld: u64) -> (InstructionResult, Gas) {
    run_on(SPEC, code, gas_limit, withheld)
}

/// [`run`] on `spec`.
fn run_on(spec: SpecId, code: &[u8], gas_limit: u64, withheld: u64) -> (InstructionResult, Gas) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl::default(),
        false,
        spec,
        gas_limit,
    );
    interpreter.gas.withhold(withheld);
    let mut host = DummyHost::new(spec);
    let table = instruction_table::<EthInterpreter, DummyHost>();
    let gas = gas_table_spec(spec);
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

/// Before EIP-150 a call forwards the gas it asks for, without the 63/64 cap: a forward the frame
/// cannot pay fails the instruction out of gas. The forward draws the withheld part first, so it
/// fails only beyond the whole regular gas, and the record holds both parts.
#[test]
fn test_a_failed_forward_records_the_regular_gas_left() {
    // `DELEGATECALL(u32::MAX, 0, 0, 0, 0, 0)`: the pushes cost 18.
    let mut code = [PUSH1, 0].repeat(4);
    code.push(PUSH20);
    code.extend_from_slice(&[0; 20]);
    code.extend_from_slice(&[PUSH4, 0xff, 0xff, 0xff, 0xff, DELEGATECALL, STOP]);
    let call_gas = u64::from(gas_table_spec(SpecId::HOMESTEAD)[DELEGATECALL as usize]);

    for withheld in [0, 500] {
        let (result, gas) = run_on(SpecId::HOMESTEAD, &code, 1_000, withheld);

        assert_eq!(result, InstructionResult::OutOfGas, "withheld {withheld}");
        assert_eq!(
            gas.failed_charge_remaining(),
            Some(1_000 - 18 - call_gas),
            "withheld {withheld}"
        );
        assert_eq!(gas.withheld_crossing(), None, "withheld {withheld}");
    }
}

/// A way to halt an interpreter.
type Halt = fn(&mut Interpreter<EthInterpreter>);

/// An opcode no spec defines, which the test below gives to an instruction of its own.
const OUT_OF_GAS: u8 = 0x0c;

/// An instruction that fails out of gas without charging anything, as a consumer's instruction
/// can.
const fn out_of_gas(_: InstructionContext<'_, DummyHost, EthInterpreter>) -> InstructionExecResult {
    Err(InstructionResult::OutOfGas)
}

/// An out-of-gas halt that no failed charge recorded records the regular gas the halt zeroes,
/// both parts: here an instruction of the consumer's that fails out of gas itself.
#[test]
fn test_an_unrecorded_out_of_gas_halt_records_the_regular_gas_left() {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::from_static(&[
            PUSH0, OUT_OF_GAS, STOP,
        ]))),
        InputsImpl::default(),
        false,
        SPEC,
        1_000,
    );
    interpreter.gas.withhold(500);
    let mut table = instruction_table::<EthInterpreter, DummyHost>();
    table[OUT_OF_GAS as usize] = Instruction::new(out_of_gas);

    let action = interpreter.run_plain(&table, &gas_table_spec(SPEC), &mut DummyHost::new(SPEC));

    let InterpreterAction::Return(result) = action else {
        panic!("expected the frame to return, got {action:?}");
    };
    assert_eq!(result.result, InstructionResult::OutOfGas);
    assert_eq!(result.gas.remaining(), 0, "the halt spent all");
    // `PUSH0` spent 2; the record holds the rest, the 500 withheld included.
    assert_eq!(result.gas.failed_charge_remaining(), Some(1_000 - 2));
}

/// The interpreter's out-of-gas halts record the regular gas left before they zero it, when no
/// record is there: `halt(OutOfGas)`, `halt_oog`, and `halt_load_error` for a skipped cold load.
/// A record already there is kept, and other halts record nothing.
#[test]
fn test_the_halts_record_an_out_of_gas_only() {
    let new = || {
        Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(Bytecode::new_raw(Bytes::from_static(&[STOP]))),
            InputsImpl::default(),
            false,
            SPEC,
            1_000,
        )
    };
    let halts: [(&str, Halt, Option<u64>); 6] = [
        (
            "halt(OutOfGas)",
            |i| i.halt(InstructionResult::OutOfGas),
            Some(1_000),
        ),
        ("halt_oog", |i| i.halt_oog(), Some(1_000)),
        (
            "halt_load_error(ColdLoadSkipped)",
            |i| i.halt_load_error(LoadError::ColdLoadSkipped),
            Some(1_000),
        ),
        (
            "halt_load_error(DBError)",
            |i| i.halt_load_error(LoadError::DBError),
            None,
        ),
        ("halt_memory_oog", |i| i.halt_memory_oog(), None),
        (
            "halt(InvalidFEOpcode)",
            |i| i.halt(InstructionResult::InvalidFEOpcode),
            None,
        ),
    ];
    for (name, halt, record) in halts {
        let mut interpreter = new();
        halt(&mut interpreter);
        assert_eq!(interpreter.gas.failed_charge_remaining(), record, "{name}");

        // A record already there is the failure that caused the halt, and stays.
        let mut interpreter = new();
        interpreter.gas.set_failed_charge_remaining(Some(7));
        halt(&mut interpreter);
        assert_eq!(interpreter.gas.failed_charge_remaining(), Some(7), "{name}");
    }
}
