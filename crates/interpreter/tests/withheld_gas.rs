//! Withheld regular gas at the instructions that read or charge a frame's gas.
//!
//! Each test runs a frame whose [`Gas`] holds part of its regular gas back from regular charges
//! ([`Gas::withhold`]). Every reader of the frame's gas that decides something other than whether
//! a regular charge can be paid must see the total: `GAS`, the gas `CALL` and `CREATE` forward,
//! the `SSTORE` stipend sentry and the skip-cold-load checks. Regular charges draw the spendable
//! part alone, and one the withheld part would have paid fails as usual but leaves a
//! [`WithheldCrossing`](revm_interpreter::gas::WithheldCrossing) behind.

use bytecode::{opcode::*, Bytecode};
use context_interface::{
    cfg::{GasParams, StateGasSite},
    host::LoadError,
    journaled_state::AccountInfoLoad,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, B256, U256,
};
use revm_interpreter::{
    host::DummyHost,
    instruction_table,
    instructions::gas_table_spec,
    interpreter::{EthInterpreter, ExtBytecode},
    FrameInput, Gas, Host, InputsImpl, InstructionResult, Interpreter, InterpreterAction,
    SStoreResult, SelfDestructResult, SharedMemory, StateLoad,
};

/// The spec every test runs at: past the `SSTORE` sentry (Istanbul), the cold access charges
/// (Berlin) and the 63/64 forwarding rule (Tangerine), and short of EIP-8037.
const SPEC: SpecId = SpecId::OSAKA;

/// Account whose code the interpreter runs.
const CONTRACT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Account the `CALL` tests call.
const CALLEE: Address = address!("0x00000000000000000000000000000000000ca11e");

/// A host whose accounts and slots are all cold, and that records whether each load was made or
/// skipped because the frame could not pay for it.
#[derive(Debug)]
struct ColdHost {
    gas_params: GasParams,
    /// Loads made.
    loaded: usize,
    /// Loads skipped at the frame's request.
    skipped: usize,
}

impl ColdHost {
    fn new() -> Self {
        Self {
            gas_params: GasParams::new_spec(SPEC),
            loaded: 0,
            skipped: 0,
        }
    }

    const fn load(&mut self, skip_cold_load: bool) -> Result<(), LoadError> {
        if skip_cold_load {
            self.skipped += 1;
            return Err(LoadError::ColdLoadSkipped);
        }
        self.loaded += 1;
        Ok(())
    }
}

impl Host for ColdHost {
    fn basefee(&self) -> U256 {
        U256::ZERO
    }

    fn blob_gasprice(&self) -> U256 {
        U256::ZERO
    }

    fn gas_limit(&self) -> U256 {
        U256::ZERO
    }

    fn difficulty(&self) -> U256 {
        U256::ZERO
    }

    fn prevrandao(&self) -> Option<U256> {
        None
    }

    fn block_number(&self) -> U256 {
        U256::ZERO
    }

    fn timestamp(&self) -> U256 {
        U256::ZERO
    }

    fn beneficiary(&self) -> Address {
        Address::ZERO
    }

    fn slot_num(&self) -> U256 {
        U256::ZERO
    }

    fn chain_id(&self) -> U256 {
        U256::ZERO
    }

    fn effective_gas_price(&self) -> U256 {
        U256::ZERO
    }

    fn caller(&self) -> Address {
        Address::ZERO
    }

    fn blob_hash(&self, _number: usize) -> Option<U256> {
        None
    }

    fn max_initcode_size(&self) -> usize {
        usize::MAX
    }

    fn gas_params(&self) -> &GasParams {
        &self.gas_params
    }

    fn is_amsterdam_eip8037_enabled(&self) -> bool {
        false
    }

    fn state_gas_price(
        &mut self,
        id: context_interface::cfg::GasId,
        _site: StateGasSite,
    ) -> Option<u64> {
        Some(self.gas_params.get(id))
    }

    fn block_hash(&mut self, _number: u64) -> Option<B256> {
        None
    }

    fn selfdestruct(
        &mut self,
        _address: Address,
        _target: Address,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
        self.load(skip_cold_load)?;
        Ok(StateLoad::new(SelfDestructResult::default(), true))
    }

    fn log(&mut self, _log: Log) {}

    fn sstore_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        value: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError> {
        self.load(skip_cold_load)?;
        let result = SStoreResult {
            original_value: StorageValue::ZERO,
            present_value: StorageValue::ZERO,
            new_value: value,
        };
        Ok(StateLoad::new(result, true))
    }

    fn sload_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, LoadError> {
        self.load(skip_cold_load)?;
        Ok(StateLoad::new(StorageValue::ZERO, true))
    }

    fn tstore(&mut self, _address: Address, _key: StorageKey, _value: StorageValue) {}

    fn tload(&mut self, _address: Address, _key: StorageKey) -> StorageValue {
        StorageValue::ZERO
    }

    fn load_account_info_skip_cold_load(
        &mut self,
        _address: Address,
        _load_code: bool,
        skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError> {
        self.load(skip_cold_load)?;
        Ok(AccountInfoLoad {
            is_cold: true,
            ..Default::default()
        })
    }
}

/// Runs `code` as [`CONTRACT`] with `gas_limit`, of which `withheld` is withheld before the
/// first instruction, until it returns or asks for a new frame.
fn run<H: Host>(
    code: &[u8],
    gas_limit: u64,
    withheld: u64,
    host: &mut H,
) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl {
            target_address: CONTRACT,
            ..Default::default()
        },
        false,
        SPEC,
        gas_limit,
    );
    interpreter.gas.withhold(withheld);
    assert_eq!(interpreter.gas.withheld(), withheld);
    let table = instruction_table::<EthInterpreter, H>();
    let gas = gas_table_spec(SPEC);
    let action = interpreter.run_plain(&table, &gas, host);
    (interpreter, action)
}

/// Runs `code` against a host on which every account and slot is warm.
fn run_warm(
    code: &[u8],
    gas_limit: u64,
    withheld: u64,
) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    run(code, gas_limit, withheld, &mut DummyHost::new(SPEC))
}

/// The result and gas of a frame that returned.
fn returned(action: &InterpreterAction) -> (InstructionResult, Gas) {
    match action {
        InterpreterAction::Return(result) => (result.result, result.gas),
        other => panic!("expected the frame to return, got {other:?}"),
    }
}

/// The input of the child frame the frame asked for.
fn new_frame(action: &InterpreterAction) -> &FrameInput {
    match action {
        InterpreterAction::NewFrame(input) => input,
        other => panic!("expected a new frame, got {other:?}"),
    }
}

/// The withheld part the frame's crossing record holds, if any.
fn crossing(gas: &Gas) -> Option<u64> {
    gas.withheld_crossing().map(|crossing| crossing.withheld())
}

fn push_address(code: &mut Vec<u8>, address: Address) {
    code.push(PUSH20);
    code.extend_from_slice(address.as_slice());
}

/// `GAS` pushes the total, the figure it pushes with nothing withheld.
#[test]
fn test_gas_pushes_the_total() {
    let code = [GAS, STOP];

    let (plain, _) = run_warm(&code, 10_000, 0);
    let (withheld, action) = run_warm(&code, 10_000, 9_000);

    assert_eq!(returned(&action).0, InstructionResult::Stop);
    assert_eq!(plain.stack.data()[0], U256::from(9_998));
    assert_eq!(withheld.stack.data()[0], plain.stack.data()[0]);
}

/// `CALL(gas, CALLEE, 0, 0, 0, 0, 0)`.
fn call(gas: U256) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0];
    push_address(&mut code, CALLEE);
    code.push(PUSH32);
    code.extend_from_slice(&gas.to_be_bytes::<32>());
    code.push(CALL);
    code
}

/// The gas limit of the child the frame asked for, and the frame's gas after the forward.
fn forwarded(action: &InterpreterAction, interpreter: &Interpreter<EthInterpreter>) -> (u64, Gas) {
    let gas_limit = match new_frame(action) {
        FrameInput::Call(inputs) => inputs.gas_limit,
        FrameInput::Create(inputs) => inputs.gas_limit(),
        FrameInput::Empty => panic!("an empty frame input"),
    };
    (gas_limit, interpreter.gas)
}

/// `CALL` with all gas forwards 63/64 of the total, drawing the withheld part first.
#[test]
fn test_call_forwards_63_64_of_the_total() {
    let code = call(U256::MAX);

    let (plain, plain_action) = run_warm(&code, 100_000, 0);
    let (withheld, action) = run_warm(&code, 100_000, 90_000);

    let (plain_limit, plain_gas) = forwarded(&plain_action, &plain);
    let (limit, gas) = forwarded(&action, &withheld);
    assert_eq!(limit, plain_limit);
    let before_forward = limit + gas.remaining();
    assert_eq!(limit, before_forward - before_forward / 64);
    assert!(limit > 90_000, "more than the spendable part is forwarded");
    // The frame keeps what it keeps with nothing withheld, all of it spendable: the forward
    // drew the withheld part first.
    assert_eq!(gas.remaining(), plain_gas.remaining());
    assert_eq!(gas.withheld(), 0);
    assert_eq!(gas.withheld_crossing(), None);
}

/// An explicit `CALL` gas argument is clamped against the total, not the spendable part.
#[test]
fn test_call_clamps_an_explicit_gas_argument_against_the_total() {
    let code = call(U256::from(50_000));

    let (plain, plain_action) = run_warm(&code, 100_000, 0);
    let (withheld, action) = run_warm(&code, 100_000, 99_000);

    let (plain_limit, plain_gas) = forwarded(&plain_action, &plain);
    let (limit, gas) = forwarded(&action, &withheld);
    assert_eq!(plain_limit, 50_000);
    assert_eq!(limit, 50_000, "above the spendable part, below the total");
    assert_eq!(gas.remaining(), plain_gas.remaining());
    assert_eq!(gas.withheld(), 99_000 - 50_000);
    assert_eq!(gas.spendable(), plain_gas.remaining() - gas.withheld());
}

/// `CREATE` forwards 63/64 of the total after its own charge, which the spendable part pays.
#[test]
fn test_create_forwards_63_64_of_the_total() {
    let code = [PUSH0, PUSH0, PUSH0, CREATE];

    let (plain, plain_action) = run_warm(&code, 100_000, 0);
    let (withheld, action) = run_warm(&code, 100_000, 50_000);

    let (plain_limit, plain_gas) = forwarded(&plain_action, &plain);
    let (limit, gas) = forwarded(&action, &withheld);
    assert_eq!(limit, plain_limit);
    let before_forward = limit + gas.remaining();
    assert_eq!(limit, before_forward - before_forward / 64);
    assert!(limit > 50_000, "more than the spendable part is forwarded");
    assert_eq!(gas.remaining(), plain_gas.remaining());
    assert_eq!(gas.withheld(), 0);
}

/// The `SSTORE` stipend sentry compares the total with the stipend: it lets a write through when
/// only the spendable part is at or below 2,300.
#[test]
fn test_sstore_sentry_reads_the_total() {
    let code = [PUSH0, PUSH0, SSTORE, STOP];

    // The sentry fires on a frame whose total is at the stipend...
    let (_, action) = run_warm(&code, 2_304, 0);
    assert_eq!(returned(&action).0, InstructionResult::ReentrancySentryOOG);

    // ...and not on one whose spendable part is.
    let (_, action) = run_warm(&code, 10_000, 7_700);
    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::Stop);
    assert!(gas.spendable() <= 2_300 && gas.remaining() > 2_300);
}

/// The `SLOAD` skip-cold check compares the total with the cold cost: the slot is loaded when
/// only the spendable part is short of it, and the cold charge then fails as a crossing.
#[test]
fn test_sload_skip_cold_check_reads_the_total() {
    let code = [PUSH0, SLOAD, STOP];

    // With nothing withheld a frame short of the cold cost skips the load.
    let mut host = ColdHost::new();
    let (_, action) = run(&code, 1_000, 0, &mut host);
    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!((host.loaded, host.skipped), (0, 1));
    assert_eq!(crossing(&gas), None);

    // With the same spendable part and enough withheld to pay, the slot is loaded.
    let mut host = ColdHost::new();
    let (_, action) = run(&code, 10_000, 9_000, &mut host);
    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!((host.loaded, host.skipped), (1, 0));
    // The record holds the withheld part the halt zeroed.
    assert_eq!(crossing(&gas), Some(9_000));
    assert_eq!(
        (gas.spendable(), gas.withheld()),
        (0, 0),
        "the halt spent all"
    );
}

/// The account skip-cold check (`BALANCE` here) behaves the same way.
#[test]
fn test_balance_skip_cold_check_reads_the_total() {
    let mut code = Vec::new();
    push_address(&mut code, CALLEE);
    code.extend_from_slice(&[BALANCE, STOP]);

    let mut host = ColdHost::new();
    let (_, action) = run(&code, 1_000, 0, &mut host);
    assert_eq!(returned(&action).0, InstructionResult::OutOfGas);
    assert_eq!((host.loaded, host.skipped), (0, 1));

    let mut host = ColdHost::new();
    let (_, action) = run(&code, 10_000, 9_000, &mut host);
    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!((host.loaded, host.skipped), (1, 0));
    assert_eq!(crossing(&gas), Some(9_000));
}

/// A memory expansion the total could pay but the spendable part cannot halts `MemoryOOG` and
/// leaves a crossing.
#[test]
fn test_mload_expansion_within_the_withheld_part_records_a_crossing() {
    // PUSH0 (2) and MLOAD (3) leave 2 spendable against the 3 one word of memory costs.
    let (_, action) = run_warm(&[PUSH0, MLOAD, STOP], 100, 93);

    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::MemoryOOG);
    assert_eq!(crossing(&gas), Some(93));
    assert_eq!(
        (gas.spendable(), gas.withheld()),
        (2, 93),
        "nothing is spent"
    );
}

/// A memory expansion beyond the total halts `MemoryOOG` without a crossing.
#[test]
fn test_mload_expansion_beyond_the_total_records_nothing() {
    let (_, action) = run_warm(&[PUSH4, 0x00, 0x10, 0x00, 0x00, MLOAD, STOP], 100, 50);

    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::MemoryOOG);
    assert_eq!(crossing(&gas), None);
}

/// An offset above `usize` fails before any charge is attempted, so there is nothing to record.
#[test]
fn test_mload_operand_above_usize_records_nothing() {
    let mut code = vec![PUSH32];
    code.extend_from_slice(&[0xff; 32]);
    code.extend_from_slice(&[MLOAD, STOP]);

    let (_, action) = run_warm(&code, 100, 50);

    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::InvalidOperandOOG);
    assert_eq!(crossing(&gas), None);
}

/// The step loop's static-gas charge records a crossing when it fails within the withheld part,
/// and nothing when it fails beyond the total.
#[test]
fn test_static_gas_failure_within_the_withheld_part_records_a_crossing() {
    let (_, action) = run_warm(&[PUSH0, STOP], 100, 99);
    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::OutOfGas);
    // The halt zeroed the withheld part; the record still holds it.
    assert_eq!(crossing(&gas), Some(99));
    assert_eq!(
        (gas.spendable(), gas.withheld()),
        (0, 0),
        "the halt spent all"
    );

    let (_, action) = run_warm(&[PUSH0, STOP], 1, 1);
    let (result, gas) = returned(&action);
    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(crossing(&gas), None);
}
