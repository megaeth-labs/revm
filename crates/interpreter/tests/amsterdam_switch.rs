//! Amsterdam opcode switch: DUPN / SWAPN / EXCHANGE / SLOTNUM below `SpecId::AMSTERDAM`.
//!
//! Default (switch off) keeps the spec-only gate. Switch on at Osaka must match Amsterdam
//! stack effect and remaining gas. These switch-on cases fail if the host flag is ignored.

use bytecode::{opcode::*, Bytecode};
use primitives::{hardfork::SpecId, Bytes, U256};
use revm_interpreter::{
    gas_table,
    host::DummyHost,
    instruction_table,
    interpreter::{EthInterpreter, ExtBytecode},
    InputsImpl, InstructionResult, Interpreter, InterpreterAction, SharedMemory,
};

const GAS_LIMIT: u64 = 10_000;

/// `DUPN 0x80` after two pushes and fifteen `DUP1`s. Stack length 18; gas 54.
const DUPN_VECTOR: &[u8] = &[
    PUSH1, 0x01, PUSH1, 0x00, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1,
    DUP1, DUP1, DUP1, DUP1, DUPN, 0x80,
];
/// Remaining gas after [`DUPN_VECTOR`] at [`GAS_LIMIT`].
const DUPN_REMAINING: u64 = 9_946;

/// `SWAPN 0x80` after two pushes, fifteen `DUP1`s and `PUSH1 0x02`. Gas 57.
const SWAPN_VECTOR: &[u8] = &[
    PUSH1, 0x01, PUSH1, 0x00, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1, DUP1,
    DUP1, DUP1, DUP1, DUP1, PUSH1, 0x02, SWAPN, 0x80,
];
const SWAPN_REMAINING: u64 = 9_943;

/// `EXCHANGE 0x8E` after three pushes. Gas 12.
const EXCHANGE_VECTOR: &[u8] = &[PUSH1, 0x00, PUSH1, 0x01, PUSH1, 0x02, EXCHANGE, 0x8E];
const EXCHANGE_REMAINING: u64 = 9_988;

const SLOTNUM_VECTOR: &[u8] = &[SLOTNUM];
const SLOTNUM_REMAINING: u64 = 9_998;

fn run(
    code: &[u8],
    spec: SpecId,
    enable_amsterdam_opcodes: bool,
) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl::default(),
        false,
        spec,
        GAS_LIMIT,
    );
    let table = instruction_table::<EthInterpreter, DummyHost>();
    let gas = gas_table();
    let mut host = DummyHost::new(spec).with_amsterdam_opcodes(enable_amsterdam_opcodes);
    let action = interpreter.run_plain(&table, &gas, &mut host);
    (interpreter, action)
}

fn assert_not_activated(code: &[u8]) {
    let (_interpreter, action) = run(code, SpecId::OSAKA, false);
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::NotActivated)
    );
}

fn assert_osaka_switch_matches_amsterdam(code: &[u8], remaining: u64) {
    let (with_switch, switch_action) = run(code, SpecId::OSAKA, true);
    let (on_amsterdam, amsterdam_action) = run(code, SpecId::AMSTERDAM, false);

    assert_eq!(
        switch_action.instruction_result(),
        Some(InstructionResult::Stop)
    );
    assert_eq!(
        amsterdam_action.instruction_result(),
        Some(InstructionResult::Stop)
    );
    assert_eq!(with_switch.stack.data(), on_amsterdam.stack.data());
    assert_eq!(with_switch.gas.remaining(), on_amsterdam.gas.remaining());
    assert_eq!(with_switch.gas.remaining(), remaining);
    assert_eq!(on_amsterdam.gas.remaining(), remaining);
}

#[test]
fn test_dupn_not_activated_on_osaka_when_switch_off() {
    assert_not_activated(DUPN_VECTOR);
}

#[test]
fn test_swapn_not_activated_on_osaka_when_switch_off() {
    assert_not_activated(SWAPN_VECTOR);
}

#[test]
fn test_exchange_not_activated_on_osaka_when_switch_off() {
    assert_not_activated(EXCHANGE_VECTOR);
}

#[test]
fn test_slotnum_not_activated_on_osaka_when_switch_off() {
    assert_not_activated(SLOTNUM_VECTOR);
}

#[test]
fn test_dupn_on_osaka_with_switch_matches_amsterdam() {
    assert_osaka_switch_matches_amsterdam(DUPN_VECTOR, DUPN_REMAINING);
    let (with_switch, _) = run(DUPN_VECTOR, SpecId::OSAKA, true);
    assert_eq!(with_switch.stack.len(), 18);
    assert_eq!(with_switch.stack.data()[17], U256::from(1));
    assert_eq!(with_switch.stack.data()[0], U256::from(1));
}

#[test]
fn test_swapn_on_osaka_with_switch_matches_amsterdam() {
    assert_osaka_switch_matches_amsterdam(SWAPN_VECTOR, SWAPN_REMAINING);
    let (with_switch, _) = run(SWAPN_VECTOR, SpecId::OSAKA, true);
    assert_eq!(with_switch.stack.len(), 18);
    assert_eq!(with_switch.stack.data()[17], U256::from(1));
    assert_eq!(with_switch.stack.data()[0], U256::from(2));
}

#[test]
fn test_exchange_on_osaka_with_switch_matches_amsterdam() {
    assert_osaka_switch_matches_amsterdam(EXCHANGE_VECTOR, EXCHANGE_REMAINING);
    let (with_switch, _) = run(EXCHANGE_VECTOR, SpecId::OSAKA, true);
    assert_eq!(with_switch.stack.len(), 3);
    assert_eq!(with_switch.stack.data()[2], U256::from(2));
    assert_eq!(with_switch.stack.data()[1], U256::from(0));
    assert_eq!(with_switch.stack.data()[0], U256::from(1));
}

#[test]
fn test_slotnum_on_osaka_with_switch_matches_amsterdam() {
    assert_osaka_switch_matches_amsterdam(SLOTNUM_VECTOR, SLOTNUM_REMAINING);
    let (with_switch, _) = run(SLOTNUM_VECTOR, SpecId::OSAKA, true);
    assert_eq!(with_switch.stack.len(), 1);
    assert_eq!(with_switch.stack.data()[0], U256::ZERO);
}

#[test]
fn test_amsterdam_with_switch_off_keeps_dupn_remaining_gas() {
    let (interpreter, action) = run(DUPN_VECTOR, SpecId::AMSTERDAM, false);
    assert_eq!(action.instruction_result(), Some(InstructionResult::Stop));
    assert_eq!(interpreter.gas.remaining(), DUPN_REMAINING);
    assert_eq!(interpreter.stack.len(), 18);
}
