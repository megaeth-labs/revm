//! Amsterdam opcode activation below `SpecId::AMSTERDAM` via the instruction table.
//!
//! The default table keeps the spec-only gate. Installing the ungated variants
//! at Osaka must match Amsterdam stack effect and remaining gas.

use bytecode::{opcode::*, Bytecode};
use primitives::{hardfork::SpecId, Bytes, U256};
use revm_interpreter::{
    enable_amsterdam_opcodes, gas_table,
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

/// A non-zero slot number so a constant-zero `Host::slot_num` would fail the installer test.
const SLOT_NUM: U256 = U256::from_limbs([7, 0, 0, 0]);

fn run(
    code: &[u8],
    spec: SpecId,
    install: bool,
    host: &mut DummyHost,
) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl::default(),
        false,
        spec,
        GAS_LIMIT,
    );
    let mut table = instruction_table::<EthInterpreter, DummyHost>();
    if install {
        enable_amsterdam_opcodes(&mut table);
    }
    let gas = gas_table();
    let action = interpreter.run_plain(&table, &gas, host);
    (interpreter, action)
}

fn run_default(code: &[u8], spec: SpecId) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    run(code, spec, false, &mut DummyHost::new(spec))
}

fn run_installed(code: &[u8], spec: SpecId) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    run(code, spec, true, &mut DummyHost::new(spec))
}

fn assert_not_activated(code: &[u8]) {
    let (_interpreter, action) = run_default(code, SpecId::OSAKA);
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::NotActivated)
    );
}

fn assert_osaka_installer_matches_amsterdam(code: &[u8], remaining: u64) {
    let (with_installer, installer_action) = run_installed(code, SpecId::OSAKA);
    let (on_amsterdam, amsterdam_action) = run_default(code, SpecId::AMSTERDAM);

    assert_eq!(
        installer_action.instruction_result(),
        Some(InstructionResult::Stop)
    );
    assert_eq!(
        amsterdam_action.instruction_result(),
        Some(InstructionResult::Stop)
    );
    assert_eq!(with_installer.stack.data(), on_amsterdam.stack.data());
    assert_eq!(with_installer.gas.remaining(), on_amsterdam.gas.remaining());
    assert_eq!(with_installer.gas.remaining(), remaining);
    assert_eq!(on_amsterdam.gas.remaining(), remaining);
}

#[test]
fn test_dupn_not_activated_on_osaka_with_default_table() {
    assert_not_activated(DUPN_VECTOR);
}

#[test]
fn test_swapn_not_activated_on_osaka_with_default_table() {
    assert_not_activated(SWAPN_VECTOR);
}

#[test]
fn test_exchange_not_activated_on_osaka_with_default_table() {
    assert_not_activated(EXCHANGE_VECTOR);
}

#[test]
fn test_slotnum_not_activated_on_osaka_with_default_table() {
    assert_not_activated(SLOTNUM_VECTOR);
}

#[test]
fn test_dupn_on_osaka_with_installer_matches_amsterdam() {
    assert_osaka_installer_matches_amsterdam(DUPN_VECTOR, DUPN_REMAINING);
    let (with_installer, _) = run_installed(DUPN_VECTOR, SpecId::OSAKA);
    assert_eq!(with_installer.stack.len(), 18);
    assert_eq!(with_installer.stack.data()[17], U256::from(1));
    assert_eq!(with_installer.stack.data()[0], U256::from(1));
}

#[test]
fn test_swapn_on_osaka_with_installer_matches_amsterdam() {
    assert_osaka_installer_matches_amsterdam(SWAPN_VECTOR, SWAPN_REMAINING);
    let (with_installer, _) = run_installed(SWAPN_VECTOR, SpecId::OSAKA);
    assert_eq!(with_installer.stack.len(), 18);
    assert_eq!(with_installer.stack.data()[17], U256::from(1));
    assert_eq!(with_installer.stack.data()[0], U256::from(2));
}

#[test]
fn test_exchange_on_osaka_with_installer_matches_amsterdam() {
    assert_osaka_installer_matches_amsterdam(EXCHANGE_VECTOR, EXCHANGE_REMAINING);
    let (with_installer, _) = run_installed(EXCHANGE_VECTOR, SpecId::OSAKA);
    assert_eq!(with_installer.stack.len(), 3);
    assert_eq!(with_installer.stack.data()[2], U256::from(2));
    assert_eq!(with_installer.stack.data()[1], U256::from(0));
    assert_eq!(with_installer.stack.data()[0], U256::from(1));
}

#[test]
fn test_slotnum_on_osaka_with_installer_matches_amsterdam() {
    assert_osaka_installer_matches_amsterdam(SLOTNUM_VECTOR, SLOTNUM_REMAINING);
    let (with_installer, _) = run_installed(SLOTNUM_VECTOR, SpecId::OSAKA);
    assert_eq!(with_installer.stack.len(), 1);
    assert_eq!(with_installer.stack.data()[0], U256::ZERO);
}

#[test]
fn test_slotnum_on_osaka_with_installer_pushes_host_slot_num() {
    let mut osaka_host = DummyHost::new(SpecId::OSAKA).with_slot_num(SLOT_NUM);
    let mut amsterdam_host = DummyHost::new(SpecId::AMSTERDAM).with_slot_num(SLOT_NUM);

    let (with_installer, installer_action) =
        run(SLOTNUM_VECTOR, SpecId::OSAKA, true, &mut osaka_host);
    let (on_amsterdam, amsterdam_action) = run(
        SLOTNUM_VECTOR,
        SpecId::AMSTERDAM,
        false,
        &mut amsterdam_host,
    );

    assert_eq!(
        installer_action.instruction_result(),
        Some(InstructionResult::Stop)
    );
    assert_eq!(
        amsterdam_action.instruction_result(),
        Some(InstructionResult::Stop)
    );
    assert_eq!(with_installer.stack.data(), on_amsterdam.stack.data());
    assert_eq!(with_installer.stack.data()[0], SLOT_NUM);
    assert_eq!(with_installer.gas.remaining(), SLOTNUM_REMAINING);
    assert_eq!(on_amsterdam.gas.remaining(), SLOTNUM_REMAINING);
}

#[test]
fn test_amsterdam_with_default_table_keeps_dupn_remaining_gas() {
    let (interpreter, action) = run_default(DUPN_VECTOR, SpecId::AMSTERDAM);
    assert_eq!(action.instruction_result(), Some(InstructionResult::Stop));
    assert_eq!(interpreter.gas.remaining(), DUPN_REMAINING);
    assert_eq!(interpreter.stack.len(), 18);
}
