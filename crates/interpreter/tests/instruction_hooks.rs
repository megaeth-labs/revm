//! The instruction hooks, at the instructions that call them.
//!
//! `SSTORE`, `LOG0`–`LOG4` and `SELFDESTRUCT` hand the write they completed to
//! [`Host::write_completed`] after their last charge, and apply its verdict. The call and creation
//! instructions hand the frame they are about to start to [`Host::frame_starting`] once the forward
//! is taken and before the frame's input is built. Each test runs one frame on [`HookHost`], which
//! records every hook call with the gas the hook was handed, charges history gas on that gas if
//! told to, and answers as the test tells it.

use bytecode::{opcode::*, Bytecode};
use context_interface::{
    cfg::{GasId, GasParams, StateGasSite},
    context::{SStoreResult, SelfDestructResult, StateLoad},
    host::{CompletedWrite, FrameStart, FrameStartKind, LoadError, WriteVerdict},
    journaled_state::AccountInfoLoad,
};
use primitives::{
    address, constants::CALL_STACK_LIMIT, hardfork::SpecId, Address, Bytes, Log, StorageKey,
    StorageValue, B256, KECCAK_EMPTY, U256,
};
use revm_interpreter::{
    host::DummyHost,
    instruction_table,
    instructions::gas_table_spec,
    interpreter::{EthInterpreter, ExtBytecode},
    FrameInput, Gas, GasTracker, Host, InputsImpl, InstructionResult, Interpreter,
    InterpreterAction, SharedMemory,
};
use state::AccountInfo;
use std::borrow::Cow;

/// The account the frame runs as.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// The frame's caller.
const CALLER: Address = address!("0x00000000000000000000000000000000000ca11e");
/// The account a call or `SELFDESTRUCT` names.
const TARGET: Address = address!("0x0000000000000000000000000000000000007a59");
/// The value the frame was called with, which `DELEGATECALL` passes on without transferring it.
const APPARENT_VALUE: u64 = 7;
/// Gas enough for every frame here.
const AMPLE: u64 = 1_000_000;
/// What a reverting hook returns.
const STOP_OUTPUT: &[u8] = b"stop";

/// A write the hook saw, owned.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Write {
    Sstore(Address, StorageKey, SStoreResult),
    Log(Address, u8, u64),
    SelfDestruct(Address, Address, SelfDestructResult),
}

impl Write {
    fn of(write: CompletedWrite<'_>) -> Self {
        match write {
            CompletedWrite::Sstore {
                address,
                key,
                result,
            } => Self::Sstore(address, key, result.clone()),
            CompletedWrite::Log {
                address,
                topics,
                data_len,
            } => Self::Log(address, topics, data_len),
            CompletedWrite::SelfDestruct {
                address,
                target,
                result,
            } => Self::SelfDestruct(address, target, result.clone()),
        }
    }
}

/// A [`DummyHost`] whose state answers as configured, and whose hooks record each call with the
/// gas they were handed, charge [`HookHost::charge`] of history gas on it, and answer
/// [`HookHost::verdict`] or [`HookHost::admit`].
struct HookHost {
    dummy: DummyHost,
    /// Every account and slot is cold, and a cold load the instruction asks to skip is refused.
    cold: bool,
    /// Every account is empty.
    empty: bool,
    /// Every account's balance.
    balance: U256,
    /// Every account's nonce.
    nonce: u64,
    /// The original and present value of every slot.
    slot: StorageValue,
    /// What `SELFDESTRUCT` finds.
    destruct: SelfDestructResult,
    /// Every state-gas price lookup fails.
    price_fails: bool,
    /// History gas each hook charges on the gas it is handed before it answers.
    charge: u64,
    verdict: WriteVerdict,
    admit: bool,
    /// Every write the hook saw, with the gas as the hook found it.
    writes: Vec<(Write, GasTracker)>,
    /// Every start the hook saw, with the gas as the hook found it.
    starts: Vec<(FrameStart, GasTracker)>,
}

impl HookHost {
    fn new(spec: SpecId) -> Self {
        Self {
            dummy: DummyHost::new(spec),
            cold: false,
            empty: false,
            balance: U256::from(AMPLE),
            nonce: 0,
            slot: StorageValue::ZERO,
            destruct: SelfDestructResult::default(),
            price_fails: false,
            charge: 0,
            verdict: WriteVerdict::Continue,
            admit: true,
            writes: Vec::new(),
            starts: Vec::new(),
        }
    }

    const fn cold(mut self) -> Self {
        self.cold = true;
        self
    }

    const fn empty(mut self) -> Self {
        self.empty = true;
        self
    }

    fn with_balance(mut self, balance: u64) -> Self {
        self.balance = U256::from(balance);
        self
    }

    const fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    fn holding(mut self, slot: u64) -> Self {
        self.slot = StorageValue::from(slot);
        self
    }

    /// `SELFDESTRUCT` of an account with a balance in favour of an account that does not exist.
    const fn destructs_into_a_new_account(mut self) -> Self {
        self.destruct = SelfDestructResult {
            had_value: true,
            target_exists: false,
            previously_destroyed: false,
        };
        self
    }

    const fn price_fails(mut self) -> Self {
        self.price_fails = true;
        self
    }

    const fn charging(mut self, charge: u64) -> Self {
        self.charge = charge;
        self
    }

    fn answering(mut self, verdict: WriteVerdict) -> Self {
        self.verdict = verdict;
        self
    }

    const fn refusing(mut self) -> Self {
        self.admit = false;
        self
    }

    const fn refuses_cold(&self, skip_cold_load: bool) -> Result<(), LoadError> {
        if self.cold && skip_cold_load {
            Err(LoadError::ColdLoadSkipped)
        } else {
            Ok(())
        }
    }

    /// The one write the hook saw.
    fn write(&self) -> &(Write, GasTracker) {
        assert_eq!(self.writes.len(), 1, "one write: {:?}", self.writes);
        &self.writes[0]
    }

    /// The one start the hook saw.
    fn start(&self) -> &(FrameStart, GasTracker) {
        assert_eq!(self.starts.len(), 1, "one start: {:?}", self.starts);
        &self.starts[0]
    }

    const fn reached(&self) -> bool {
        !self.writes.is_empty() || !self.starts.is_empty()
    }
}

impl Host for HookHost {
    fn write_completed(&mut self, write: CompletedWrite<'_>, gas: &mut GasTracker) -> WriteVerdict {
        self.writes.push((Write::of(write), *gas));
        if self.charge > 0 {
            let _ = gas.record_history_cost(self.charge);
        }
        self.verdict.clone()
    }

    fn frame_starting(&mut self, start: &mut FrameStart, gas: &mut GasTracker) -> bool {
        self.starts.push((*start, *gas));
        if self.charge > 0 {
            let _ = gas.record_history_cost(self.charge);
        }
        self.admit
    }

    fn state_gas_price(&mut self, id: GasId, _site: StateGasSite) -> Option<u64> {
        (!self.price_fails).then(|| self.gas_params().get(id))
    }

    fn basefee(&self) -> U256 {
        self.dummy.basefee()
    }
    fn blob_gasprice(&self) -> U256 {
        self.dummy.blob_gasprice()
    }
    fn gas_limit(&self) -> U256 {
        self.dummy.gas_limit()
    }
    fn difficulty(&self) -> U256 {
        self.dummy.difficulty()
    }
    fn prevrandao(&self) -> Option<U256> {
        self.dummy.prevrandao()
    }
    fn block_number(&self) -> U256 {
        self.dummy.block_number()
    }
    fn timestamp(&self) -> U256 {
        self.dummy.timestamp()
    }
    fn beneficiary(&self) -> Address {
        self.dummy.beneficiary()
    }
    fn slot_num(&self) -> U256 {
        self.dummy.slot_num()
    }
    fn chain_id(&self) -> U256 {
        self.dummy.chain_id()
    }
    fn effective_gas_price(&self) -> U256 {
        self.dummy.effective_gas_price()
    }
    fn caller(&self) -> Address {
        self.dummy.caller()
    }
    fn blob_hash(&self, number: usize) -> Option<U256> {
        self.dummy.blob_hash(number)
    }
    fn max_initcode_size(&self) -> usize {
        usize::MAX
    }
    fn gas_params(&self) -> &GasParams {
        self.dummy.gas_params()
    }
    fn is_amsterdam_eip8037_enabled(&self) -> bool {
        self.dummy.is_amsterdam_eip8037_enabled()
    }
    fn block_hash(&mut self, number: u64) -> Option<B256> {
        self.dummy.block_hash(number)
    }

    fn selfdestruct(
        &mut self,
        _address: Address,
        _target: Address,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
        self.refuses_cold(skip_cold_load)?;
        Ok(StateLoad::new(self.destruct.clone(), self.cold))
    }

    fn log(&mut self, log: Log) {
        self.dummy.log(log)
    }

    fn sstore_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        value: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError> {
        self.refuses_cold(skip_cold_load)?;
        let result = SStoreResult {
            original_value: self.slot,
            present_value: self.slot,
            new_value: value,
        };
        Ok(StateLoad::new(result, self.cold))
    }

    fn sload_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, LoadError> {
        self.dummy
            .sload_skip_cold_load(address, key, skip_cold_load)
    }

    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue) {
        self.dummy.tstore(address, key, value)
    }

    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue {
        self.dummy.tload(address, key)
    }

    fn load_account_info_skip_cold_load(
        &mut self,
        _address: Address,
        _load_code: bool,
        skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError> {
        self.refuses_cold(skip_cold_load)?;
        Ok(AccountInfoLoad {
            account: Cow::Owned(AccountInfo::from_balance(self.balance).with_nonce(self.nonce)),
            is_cold: self.cold,
            is_empty: self.empty,
        })
    }
}

/// Runs `code` as [`CONTRACT`], called by [`CALLER`] with [`APPARENT_VALUE`], with `gas_limit`
/// regular gas and `reservoir`, and returns what the frame asked for and its gas.
fn run(
    spec: SpecId,
    code: &[u8],
    gas_limit: u64,
    reservoir: u64,
    host: &mut HookHost,
) -> (InterpreterAction, Gas) {
    run_at_depth(spec, code, gas_limit, reservoir, 0, host)
}

/// [`run`] in a frame at `depth`.
fn run_at_depth(
    spec: SpecId,
    code: &[u8],
    gas_limit: u64,
    reservoir: u64,
    depth: usize,
    host: &mut HookHost,
) -> (InterpreterAction, Gas) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl {
            target_address: CONTRACT,
            caller_address: CALLER,
            call_value: U256::from(APPARENT_VALUE),
            depth,
            ..Default::default()
        },
        false,
        spec,
        gas_limit,
    );
    interpreter.gas = Gas::new_with_regular_gas_and_reservoir(gas_limit, reservoir);
    let table = instruction_table::<EthInterpreter, HookHost>();
    let action = interpreter.run_plain(&table, &gas_table_spec(spec), host);
    (action, interpreter.gas)
}

/// The result a frame returned with, and its result's gas.
fn returned(action: InterpreterAction) -> (InstructionResult, Bytes, Gas) {
    match action {
        InterpreterAction::Return(result) => (result.result, result.output, result.gas),
        other => panic!("expected the frame to return, got {other:?}"),
    }
}

/// The frame a frame asked to start.
fn started(action: InterpreterAction) -> FrameInput {
    match action {
        InterpreterAction::NewFrame(input) => input,
        other => panic!("expected a new frame, got {other:?}"),
    }
}

/// The static gas of `op` on `spec`.
const fn static_gas(spec: SpecId, op: u8) -> u64 {
    gas_table_spec(spec)[op as usize] as u64
}

/// `SSTORE(0, 1)`, then `STOP`.
const SSTORE_CODE: [u8; 6] = [PUSH1, 1, PUSH1, 0, SSTORE, STOP];
/// `SSTORE(0, 0)`, then `STOP`.
const CLEAR_CODE: [u8; 6] = [PUSH1, 0, PUSH1, 0, SSTORE, STOP];
/// What `SSTORE` finds when [`SSTORE_CODE`] sets a slot from zero.
fn set_from_zero() -> SStoreResult {
    SStoreResult {
        original_value: StorageValue::ZERO,
        present_value: StorageValue::ZERO,
        new_value: StorageValue::from(1),
    }
}

/// `LOG1` of `len` bytes of memory at offset 0, then `STOP`.
fn log1_code(len: u8) -> Vec<u8> {
    vec![PUSH1, 0xaa, PUSH1, len, PUSH1, 0, LOG1, STOP]
}

/// `SELFDESTRUCT` in favour of [`TARGET`].
fn selfdestruct_code() -> Vec<u8> {
    let mut code = vec![PUSH20];
    code.extend_from_slice(TARGET.as_slice());
    code.push(SELFDESTRUCT);
    code
}

/// `op` (a call) to [`TARGET`] with `gas`, `value` (for `CALL` and `CALLCODE`), `in_len` bytes of
/// input at offset 0 and no output, then `STOP`.
fn call_code(op: u8, gas: u32, value: u8, in_len: u8) -> Vec<u8> {
    let mut code = vec![PUSH1, 0, PUSH1, 0, PUSH1, in_len, PUSH1, 0];
    if matches!(op, CALL | CALLCODE) {
        code.extend_from_slice(&[PUSH1, value]);
    }
    code.push(PUSH20);
    code.extend_from_slice(TARGET.as_slice());
    code.push(PUSH4);
    code.extend_from_slice(&gas.to_be_bytes());
    code.extend_from_slice(&[op, STOP]);
    code
}

/// The gas [`call_code`]'s pushes cost.
const fn call_pushes(op: u8) -> u64 {
    if matches!(op, CALL | CALLCODE) {
        7 * 3
    } else {
        6 * 3
    }
}

/// `op` (`CREATE` or `CREATE2`) of `len` bytes of memory at offset 0, endowed with `value`, then
/// `STOP`.
fn create_code(op: u8, len: u8, value: u8) -> Vec<u8> {
    let mut code = Vec::new();
    if op == CREATE2 {
        code.extend_from_slice(&[PUSH1, 0x5a]);
    }
    code.extend_from_slice(&[PUSH1, len, PUSH1, 0, PUSH1, value, op, STOP]);
    code
}

/// The gas [`create_code`]'s pushes cost.
const fn create_pushes(op: u8) -> u64 {
    if op == CREATE2 {
        4 * 3
    } else {
        3 * 3
    }
}

/// A write instruction under test: its name, code, the host it runs on, the write the hook sees,
/// and the result of a frame one gas short of the instruction's last charge.
type WriteCase = (
    &'static str,
    Vec<u8>,
    fn(SpecId) -> HookHost,
    Write,
    InstructionResult,
);

/// The write instructions, each on a host that makes it charge everything it can: a cold slot set
/// from zero, a cold slot cleared, which earns a refund, a log of 32 bytes, and a cold
/// `SELFDESTRUCT` into a new account, which earns a refund before EIP-3529.
fn write_cases() -> Vec<WriteCase> {
    let destruct = SelfDestructResult {
        had_value: true,
        target_exists: false,
        previously_destroyed: false,
    };
    vec![
        (
            "SSTORE",
            SSTORE_CODE.to_vec(),
            |spec| HookHost::new(spec).cold(),
            Write::Sstore(CONTRACT, StorageKey::ZERO, set_from_zero()),
            InstructionResult::OutOfGas,
        ),
        (
            "SSTORE clearing a slot",
            CLEAR_CODE.to_vec(),
            |spec| HookHost::new(spec).cold().holding(1),
            Write::Sstore(
                CONTRACT,
                StorageKey::ZERO,
                SStoreResult {
                    original_value: StorageValue::from(1),
                    present_value: StorageValue::from(1),
                    new_value: StorageValue::ZERO,
                },
            ),
            InstructionResult::OutOfGas,
        ),
        (
            "LOG1",
            log1_code(32),
            HookHost::new,
            Write::Log(CONTRACT, 1, 32),
            InstructionResult::MemoryOOG,
        ),
        (
            "SELFDESTRUCT",
            selfdestruct_code(),
            |spec| HookHost::new(spec).cold().destructs_into_a_new_account(),
            Write::SelfDestruct(CONTRACT, TARGET, destruct),
            InstructionResult::OutOfGas,
        ),
    ]
}

/// Each write instruction hands its write to the hook after its last charge: the frame ends on the
/// gas the hook was handed, so nothing is charged after it, and a frame one gas short of what the
/// instruction charges before the hook fails without reaching it. On a spec with and without
/// EIP-8037, so the last charge is the state-gas charge where there is one, and on one before
/// EIP-3529, where `SELFDESTRUCT` earns a refund.
#[test]
fn test_a_write_reaches_the_hook_after_every_charge() {
    for spec in [SpecId::BERLIN, SpecId::OSAKA, SpecId::AMSTERDAM] {
        for (name, code, host, write, short) in write_cases() {
            let mut ample = host(spec);
            let (action, gas) = run(spec, &code, AMPLE, 0, &mut ample);
            let (result, _, result_gas) = returned(action);
            let (seen, at_hook) = ample.write().clone();
            assert_eq!(seen, write, "{name} on {spec:?}");
            assert!(result.is_ok(), "{name} on {spec:?}: {result:?}");
            assert_eq!(
                *result_gas.tracker(),
                at_hook,
                "{name} on {spec:?}: nothing is charged after the hook"
            );
            assert_eq!(*gas.tracker(), at_hook);
            let refunds = name == "SSTORE clearing a slot"
                || (name == "SELFDESTRUCT" && !spec.is_enabled_in(SpecId::LONDON));
            assert_eq!(
                at_hook.refunded() > 0,
                refunds,
                "{name} on {spec:?}: the refund is made before the hook"
            );

            // What the instruction charges before the hook reaches it with nothing left; one gas
            // less does not reach it.
            let needed = AMPLE - at_hook.remaining();
            let mut exact = host(spec);
            let (action, _) = run(spec, &code, needed, 0, &mut exact);
            assert!(returned(action).0.is_ok(), "{name} on {spec:?}");
            assert_eq!(exact.write().1.remaining(), 0, "{name} on {spec:?}");

            let mut one_short = host(spec);
            let (action, _) = run(spec, &code, needed - 1, 0, &mut one_short);
            assert_eq!(returned(action).0, short, "{name} on {spec:?}");
            assert!(
                !one_short.reached(),
                "{name} on {spec:?}: the hook is not reached"
            );
        }
    }
}

/// A failure before the hook: its name, the spec, the code, the host, the gas limit and the result
/// the frame fails with.
type FailureCase<Name> = (Name, SpecId, Vec<u8>, HookHost, u64, InstructionResult);

/// Every earlier failure of a write instruction, a charge or a check before its last charge, ends
/// the instruction before the hook.
#[test]
fn test_a_write_that_fails_before_its_last_charge_never_reaches_the_hook() {
    let osaka = GasParams::new_spec(SpecId::OSAKA);
    let amsterdam = GasParams::new_spec(SpecId::AMSTERDAM);
    let petersburg = GasParams::new_spec(SpecId::PETERSBURG);
    let set = set_from_zero();
    let cases: Vec<FailureCase<&str>> = vec![
        (
            "SSTORE with the stipend left",
            SpecId::OSAKA,
            SSTORE_CODE.to_vec(),
            HookHost::new(SpecId::OSAKA),
            6 + osaka.call_stipend(),
            InstructionResult::ReentrancySentryOOG,
        ),
        (
            "SSTORE's static charge, before EIP-2200's stipend check",
            SpecId::PETERSBURG,
            SSTORE_CODE.to_vec(),
            HookHost::new(SpecId::PETERSBURG),
            6 + petersburg.sstore_static_gas() - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "SSTORE's dynamic charge, before its state-gas charge",
            SpecId::AMSTERDAM,
            SSTORE_CODE.to_vec(),
            HookHost::new(SpecId::AMSTERDAM).cold(),
            6 + amsterdam.sstore_static_gas() + amsterdam.sstore_dynamic_gas(true, &set, true) - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "SSTORE's state-gas price",
            SpecId::AMSTERDAM,
            SSTORE_CODE.to_vec(),
            HookHost::new(SpecId::AMSTERDAM).price_fails(),
            AMPLE,
            InstructionResult::FatalExternalError,
        ),
        (
            "LOG1's log charge, before its memory",
            SpecId::OSAKA,
            log1_code(32),
            HookHost::new(SpecId::OSAKA),
            9 + static_gas(SpecId::OSAKA, LOG1) + osaka.log_cost(1, 32) - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "SELFDESTRUCT's cold target, refused",
            SpecId::OSAKA,
            selfdestruct_code(),
            HookHost::new(SpecId::OSAKA).cold(),
            3 + static_gas(SpecId::OSAKA, SELFDESTRUCT) + osaka.selfdestruct_cold_cost() - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "SELFDESTRUCT's charge, before its state-gas charge",
            SpecId::AMSTERDAM,
            selfdestruct_code(),
            HookHost::new(SpecId::AMSTERDAM)
                .cold()
                .destructs_into_a_new_account(),
            3 + static_gas(SpecId::AMSTERDAM, SELFDESTRUCT)
                + amsterdam.selfdestruct_cost(true, true)
                - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "SELFDESTRUCT's state-gas price",
            SpecId::AMSTERDAM,
            selfdestruct_code(),
            HookHost::new(SpecId::AMSTERDAM)
                .destructs_into_a_new_account()
                .price_fails(),
            AMPLE,
            InstructionResult::FatalExternalError,
        ),
    ];
    for (name, spec, code, mut host, gas_limit, expected) in cases {
        let (action, _) = run(spec, &code, gas_limit, 0, &mut host);
        assert_eq!(returned(action).0, expected, "{name}");
        assert!(!host.reached(), "{name}: the hook is not reached");
    }
}

/// `OutOfGas` fails the instruction as a failed charge does: the frame halts out of gas, and the
/// record of what it had left is what the hook left.
///
/// The verdict tests run on Osaka, where the write instructions charge no state gas, so the hook
/// finds the frame's reservoir of 1,000 whole: a hook charge of 1,500 spills 500 into regular gas.
#[test]
fn test_out_of_gas_fails_the_write_on_the_gas_the_hook_left() {
    for (name, code, host, _, _) in write_cases() {
        let mut host = host(SpecId::OSAKA)
            .charging(1_500)
            .answering(WriteVerdict::OutOfGas);
        let (action, _) = run(SpecId::OSAKA, &code, AMPLE, 1_000, &mut host);
        let (result, _, gas) = returned(action);
        let at_hook = host.write().1;

        assert_eq!(result, InstructionResult::OutOfGas, "{name}");
        assert_eq!(gas.remaining(), 0, "{name}: the halt spends all");
        assert_eq!(
            gas.failed_charge_remaining(),
            Some(at_hook.remaining() - 500),
            "{name}: the record is what the hook left"
        );
    }
}

/// A hook's own failed charge is the record the halt keeps.
#[test]
fn test_out_of_gas_after_a_failed_hook_charge_keeps_its_record() {
    let mut host = HookHost::new(SpecId::OSAKA)
        .charging(AMPLE)
        .answering(WriteVerdict::OutOfGas);
    let (action, _) = run(SpecId::OSAKA, &log1_code(32), AMPLE, 0, &mut host);
    let (result, _, gas) = returned(action);

    assert_eq!(result, InstructionResult::OutOfGas);
    assert_eq!(
        gas.failed_charge_remaining(),
        Some(host.write().1.remaining())
    );
}

/// `Revert` ends the frame as a `Revert` with the verdict's output, on the gas the hook left: a
/// `SELFDESTRUCT` so reverted returns `Revert`, not `SelfDestruct`.
#[test]
fn test_revert_ends_the_frame_with_the_output_on_the_gas_the_hook_left() {
    for (name, code, host, _, _) in write_cases() {
        let mut host = host(SpecId::OSAKA)
            .charging(400)
            .answering(WriteVerdict::Revert(Bytes::from_static(STOP_OUTPUT)));
        let (action, _) = run(SpecId::OSAKA, &code, AMPLE, 1_000, &mut host);
        let (result, output, gas) = returned(action);
        let mut expected = host.write().1;
        assert!(expected.record_history_cost(400));

        assert_eq!(result, InstructionResult::Revert, "{name}");
        assert_eq!(output, Bytes::from_static(STOP_OUTPUT), "{name}");
        assert_eq!(*gas.tracker(), expected, "{name}");
        assert_eq!(gas.reservoir(), 600, "{name}: the hook's charge stands");
    }
}

/// `Continue` ends the instruction as it would have ended, on the gas the hook left: a
/// `SELFDESTRUCT` self-destructs.
#[test]
fn test_continue_ends_the_write_on_the_gas_the_hook_left() {
    for ((name, code, host, _, _), ends) in write_cases().into_iter().zip([
        InstructionResult::Stop,
        InstructionResult::Stop,
        InstructionResult::Stop,
        InstructionResult::SelfDestruct,
    ]) {
        let mut host = host(SpecId::OSAKA).charging(1_500);
        let (action, _) = run(SpecId::OSAKA, &code, AMPLE, 1_000, &mut host);
        let (result, _, gas) = returned(action);
        let at_hook = host.write().1;

        assert_eq!(result, ends, "{name}");
        assert_eq!(gas.reservoir(), 0, "{name}");
        assert_eq!(gas.remaining(), at_hook.remaining() - 500, "{name}");
        assert_eq!(gas.history_gas_spent(), 1_500, "{name}");
    }
}

/// The six frame-starting instructions, each with what the hook sees of it: kind, caller, target
/// and value.
fn start_cases() -> Vec<(&'static str, Vec<u8>, FrameStart)> {
    let value = U256::from(3);
    let start = |kind, caller, target, value| FrameStart::new(kind, caller, target, value, 0);
    vec![
        (
            "CALL",
            call_code(CALL, 50_000, 3, 0),
            start(FrameStartKind::Call, CONTRACT, Some(TARGET), value),
        ),
        (
            "CALLCODE",
            call_code(CALLCODE, 50_000, 3, 0),
            start(FrameStartKind::CallCode, CONTRACT, Some(CONTRACT), value),
        ),
        (
            "DELEGATECALL",
            call_code(DELEGATECALL, 50_000, 0, 0),
            start(
                FrameStartKind::DelegateCall,
                CALLER,
                Some(CONTRACT),
                U256::ZERO,
            ),
        ),
        (
            "STATICCALL",
            call_code(STATICCALL, 50_000, 0, 0),
            start(
                FrameStartKind::StaticCall,
                CONTRACT,
                Some(TARGET),
                U256::ZERO,
            ),
        ),
        (
            "CREATE",
            create_code(CREATE, 0, 3),
            start(
                FrameStartKind::Create,
                CONTRACT,
                Some(created(CREATE)),
                value,
            ),
        ),
        (
            "CREATE2",
            create_code(CREATE2, 0, 3),
            start(
                FrameStartKind::Create2,
                CONTRACT,
                Some(created(CREATE2)),
                value,
            ),
        ),
    ]
}

/// The address [`create_code`]`(op, 0, _)` creates at, [`CONTRACT`]'s nonce being `0`: `CREATE`
/// from the nonce, `CREATE2` from its salt `0x5a` and the hash of no init code.
fn created(op: u8) -> Address {
    if op == CREATE2 {
        CONTRACT.create2(B256::from(U256::from(0x5a)), KECCAK_EMPTY)
    } else {
        CONTRACT.create(0)
    }
}

/// The gas limit and reservoir of a frame's input.
fn input_gas(input: &FrameInput) -> (u64, u64) {
    match input {
        FrameInput::Call(inputs) => (inputs.gas_limit, inputs.reservoir),
        FrameInput::Create(inputs) => (inputs.gas_limit(), inputs.reservoir()),
        FrameInput::Empty => panic!("an empty frame input"),
    }
}

/// Each frame-starting instruction hands its start to the hook once the forward is taken, and
/// charges nothing after it: the instruction leaves the gas the hook was handed, apart from the
/// byte pool, whose share goes to the frame after the hook, and the frame's input carries the
/// forward, with a value call's stipend on top.
#[test]
fn test_a_start_reaches_the_hook_with_the_forward_taken() {
    let spec = SpecId::OSAKA;
    let params = GasParams::new_spec(spec);
    for (name, code, expected) in start_cases() {
        let mut host = HookHost::new(spec);
        let (action, gas) = run(spec, &code, AMPLE, 0, &mut host);
        let input = started(action);
        let (start, at_hook) = *host.start();
        let (gas_limit, reservoir) = input_gas(&input);
        let op = code[code.len() - 2];
        let charged = if matches!(op, CREATE | CREATE2) {
            create_pushes(op) + params.create_cost()
        } else {
            let transfer = if start.value.is_zero() {
                0
            } else {
                params.transfer_value_cost()
            };
            call_pushes(op) + static_gas(spec, op) + transfer
        };
        let before_forward = AMPLE - charged;
        let forward = if matches!(op, CREATE | CREATE2) {
            before_forward - before_forward / 64
        } else {
            50_000
        };

        let expected = FrameStart::new(
            expected.kind,
            expected.caller,
            expected.target,
            expected.value,
            forward,
        );
        assert_eq!(start, expected, "{name}: the start's facts");
        assert_eq!(
            at_hook.remaining(),
            before_forward - forward,
            "{name}: the hook sees the forward taken"
        );
        let mut after_the_forward = at_hook;
        after_the_forward.set_bytes_remaining(0);
        assert_eq!(
            *gas.tracker(),
            after_the_forward,
            "{name}: nothing is charged after the hook; the whole pool is forwarded"
        );
        let stipend = if matches!(op, CALL | CALLCODE) {
            params.call_stipend()
        } else {
            0
        };
        assert_eq!(
            gas_limit,
            forward + stipend,
            "{name}: the frame's gas limit"
        );
        assert_eq!(reservoir, 0, "{name}");
    }
}

/// A creation's start carries the address the creation takes, with EIP-8037 and without it, up to
/// the frame below the depth limit, and the frame's input carries the same address, computed once:
/// the handler reuses it whatever nonce it reads.
#[test]
fn test_a_creation_start_carries_the_address_its_input_caches() {
    for spec in [SpecId::OSAKA, SpecId::AMSTERDAM] {
        for op in [CREATE, CREATE2] {
            for depth in [0, CALL_STACK_LIMIT as usize - 1] {
                let mut host = HookHost::new(spec);
                let code = create_code(op, 0, 3);
                let (action, _) = run_at_depth(spec, &code, AMPLE, 0, depth, &mut host);
                let FrameInput::Create(inputs) = started(action) else {
                    panic!("{spec:?} {op:#x} at {depth}: a creation's input");
                };

                assert_eq!(
                    host.start().0.target,
                    Some(created(op)),
                    "{spec:?} {op:#x} at {depth}"
                );
                assert_eq!(
                    inputs.created_address(7),
                    created(op),
                    "{spec:?} {op:#x} at {depth}: the input carries the address the hook saw"
                );
            }
        }
    }
}

/// Without EIP-8037, a creation the handler fails before it takes an address reaches the hook with
/// none, and its input carries none: an endowment above the caller's balance, a caller's nonce at
/// its maximum, and a frame at the depth limit. With EIP-8037 the instruction ends each before the
/// hook and pushes zero.
#[test]
fn test_a_creation_that_fails_before_its_address_starts_without_one() {
    let cases = |spec| {
        [
            (
                "an endowment above the balance",
                HookHost::new(spec).with_balance(2),
                0,
            ),
            (
                "a nonce at its maximum",
                HookHost::new(spec).with_nonce(u64::MAX),
                0,
            ),
            (
                "the depth limit",
                HookHost::new(spec),
                CALL_STACK_LIMIT as usize,
            ),
        ]
    };
    for op in [CREATE, CREATE2] {
        let code = create_code(op, 0, 3);
        for (name, mut host, depth) in cases(SpecId::OSAKA) {
            let (action, _) = run_at_depth(SpecId::OSAKA, &code, AMPLE, 0, depth, &mut host);
            let FrameInput::Create(inputs) = started(action) else {
                panic!("{op:#x}, {name}: a creation's input");
            };

            assert_eq!(host.start().0.target, None, "{op:#x}, {name}");
            if op == CREATE {
                assert_eq!(
                    inputs.created_address(5),
                    CONTRACT.create(5),
                    "{name}: the input carries no address"
                );
            }
        }
        for (name, mut host, depth) in cases(SpecId::AMSTERDAM) {
            let (action, _) = run_at_depth(SpecId::AMSTERDAM, &code, AMPLE, 0, depth, &mut host);

            assert_eq!(
                returned(action).0,
                InstructionResult::Stop,
                "{op:#x}, {name}"
            );
            assert!(!host.reached(), "{op:#x}, {name}: the hook is not reached");
        }
    }
}

/// A call's forward is the gas it names when that is below 63/64 of what the caller has left, and
/// that cap otherwise, for every kind of call.
#[test]
fn test_the_forward_is_the_named_gas_or_the_63_64_cap() {
    let spec = SpecId::OSAKA;
    for op in [CALL, CALLCODE, DELEGATECALL, STATICCALL] {
        let charged = call_pushes(op) + static_gas(spec, op);
        let left = AMPLE - charged;
        for (named, forward) in [(20_000, 20_000), (u32::MAX, left - left / 64)] {
            let mut host = HookHost::new(spec);
            let (action, _) = run(spec, &call_code(op, named, 0, 0), AMPLE, 0, &mut host);

            assert_eq!(host.start().0.forward, forward, "{op:#x}, named {named}");
            assert_eq!(
                input_gas(&started(action)).0,
                forward,
                "{op:#x}, named {named}"
            );
        }
    }
}

/// A value call's stipend is part of the frame's gas limit but not of the forward the hook sees:
/// the caller does not pay it.
#[test]
fn test_the_value_stipend_is_not_part_of_the_forward() {
    let spec = SpecId::OSAKA;
    let params = GasParams::new_spec(spec);
    for op in [CALL, CALLCODE] {
        let mut host = HookHost::new(spec);
        let (action, gas) = run(spec, &call_code(op, 20_000, 1, 0), AMPLE, 0, &mut host);
        let charged = call_pushes(op) + static_gas(spec, op) + params.transfer_value_cost();

        assert_eq!(host.start().0.forward, 20_000);
        assert_eq!(
            input_gas(&started(action)).0,
            20_000 + params.call_stipend()
        );
        assert_eq!(gas.remaining(), AMPLE - charged - 20_000);
    }
}

/// The frame's input copies the reservoir as the hook left it, for every kind of start. A charge the
/// reservoir covers leaves the forward and the caller's regular gas alone; one that spills draws the
/// caller's regular gas and empties the reservoir, and the forward is still what was taken before.
#[test]
fn test_the_input_copies_the_reservoir_the_hook_left() {
    let spec = SpecId::AMSTERDAM;
    for (name, code, _) in start_cases() {
        let mut quiet = HookHost::new(spec);
        let (quiet_action, quiet_gas) = run(spec, &code, AMPLE, 10_000, &mut quiet);
        let quiet_limit = input_gas(&started(quiet_action)).0;

        for (charge, reservoir, spilled) in [(4_000, 6_000, 0), (10_500, 0, 500)] {
            let mut host = HookHost::new(spec).charging(charge);
            let (action, gas) = run(spec, &code, AMPLE, 10_000, &mut host);
            let (gas_limit, input_reservoir) = input_gas(&started(action));

            assert_eq!(host.start().1.reservoir(), 10_000, "{name}");
            assert_eq!(input_reservoir, reservoir, "{name}, charge {charge}");
            assert_eq!(gas.reservoir(), reservoir, "{name}, charge {charge}");
            assert_eq!(
                gas_limit, quiet_limit,
                "{name}, charge {charge}: the forward"
            );
            assert_eq!(
                gas.remaining(),
                quiet_gas.remaining() - spilled,
                "{name}, charge {charge}"
            );
        }
    }
}

/// `false` fails the instruction out of gas and starts no frame. The forward stays taken, so the
/// halt records what the caller had left after it.
#[test]
fn test_a_refused_start_fails_out_of_gas_and_starts_no_frame() {
    for (name, code, _) in start_cases() {
        let mut host = HookHost::new(SpecId::OSAKA).refusing();
        let (action, _) = run(SpecId::OSAKA, &code, AMPLE, 0, &mut host);
        let (result, _, gas) = returned(action);

        assert_eq!(result, InstructionResult::OutOfGas, "{name}");
        assert_eq!(
            gas.failed_charge_remaining(),
            Some(host.start().1.remaining()),
            "{name}"
        );
    }
}

/// Every failure of a frame-starting instruction before its forward is taken ends the instruction
/// before the hook.
#[test]
fn test_a_start_that_fails_before_the_forward_never_reaches_the_hook() {
    let osaka = GasParams::new_spec(SpecId::OSAKA);
    let amsterdam = GasParams::new_spec(SpecId::AMSTERDAM);
    let call = |params: &GasParams, spec| {
        call_pushes(CALL) + static_gas(spec, CALL) + params.transfer_value_cost()
    };
    let mut cases: Vec<FailureCase<String>> = Vec::new();
    for op in [CALL, CALLCODE, DELEGATECALL, STATICCALL] {
        cases.push((
            format!("{op:#x}'s input memory"),
            SpecId::OSAKA,
            call_code(op, 1_000, 0, 32),
            HookHost::new(SpecId::OSAKA),
            call_pushes(op) + static_gas(SpecId::OSAKA, op) + 2,
            InstructionResult::MemoryOOG,
        ));
        cases.push((
            format!("{op:#x}'s cold target, refused"),
            SpecId::OSAKA,
            call_code(op, 1_000, 0, 0),
            HookHost::new(SpecId::OSAKA).cold(),
            call_pushes(op) + static_gas(SpecId::OSAKA, op) + osaka.cold_account_additional_cost()
                - 1,
            InstructionResult::OutOfGas,
        ));
    }
    cases.extend([
        (
            "CALL's value charge".to_string(),
            SpecId::OSAKA,
            call_code(CALL, 1_000, 1, 0),
            HookHost::new(SpecId::OSAKA),
            call(&osaka, SpecId::OSAKA) - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "CALL's account charge".to_string(),
            SpecId::OSAKA,
            call_code(CALL, 1_000, 1, 0),
            HookHost::new(SpecId::OSAKA).cold().empty(),
            call(&osaka, SpecId::OSAKA)
                + osaka.cold_account_additional_cost()
                + osaka.new_account_cost(true, true)
                - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "CALL's state-gas charge".to_string(),
            SpecId::AMSTERDAM,
            call_code(CALL, 1_000, 1, 0),
            HookHost::new(SpecId::AMSTERDAM).empty(),
            call(&amsterdam, SpecId::AMSTERDAM)
                + amsterdam.new_account_cost(true, true)
                + amsterdam.get(GasId::new_account_state_gas())
                - 1,
            InstructionResult::OutOfGas,
        ),
        (
            "CALL's state-gas price".to_string(),
            SpecId::AMSTERDAM,
            call_code(CALL, 1_000, 1, 0),
            HookHost::new(SpecId::AMSTERDAM).empty().price_fails(),
            AMPLE,
            InstructionResult::FatalExternalError,
        ),
        (
            "CALL's forward, before EIP-150's cap".to_string(),
            SpecId::HOMESTEAD,
            call_code(CALL, 1_000, 0, 0),
            HookHost::new(SpecId::HOMESTEAD),
            call_pushes(CALL) + static_gas(SpecId::HOMESTEAD, CALL) + 999,
            InstructionResult::OutOfGas,
        ),
    ]);
    for op in [CREATE, CREATE2] {
        let pushes = create_pushes(op);
        let cost = if op == CREATE2 {
            osaka.create2_cost(32)
        } else {
            osaka.create_cost()
        };
        let amsterdam_cost = if op == CREATE2 {
            amsterdam.create2_cost(32)
        } else {
            amsterdam.create_cost()
        };
        cases.extend([
            (
                format!("{op:#x}'s init-code charge"),
                SpecId::OSAKA,
                create_code(op, 32, 0),
                HookHost::new(SpecId::OSAKA),
                pushes + osaka.initcode_cost(32) - 1,
                InstructionResult::OutOfGas,
            ),
            (
                format!("{op:#x}'s memory"),
                SpecId::OSAKA,
                create_code(op, 32, 0),
                HookHost::new(SpecId::OSAKA),
                pushes + osaka.initcode_cost(32) + 2,
                InstructionResult::MemoryOOG,
            ),
            (
                format!("{op:#x}'s creation charge"),
                SpecId::OSAKA,
                create_code(op, 32, 0),
                HookHost::new(SpecId::OSAKA),
                pushes + osaka.initcode_cost(32) + 3 + cost - 1,
                InstructionResult::OutOfGas,
            ),
            (
                format!("{op:#x}'s state-gas charge"),
                SpecId::AMSTERDAM,
                create_code(op, 32, 0),
                HookHost::new(SpecId::AMSTERDAM).empty(),
                pushes
                    + amsterdam.initcode_cost(32)
                    + 3
                    + amsterdam_cost
                    + amsterdam.get(GasId::create_state_gas())
                    - 1,
                InstructionResult::OutOfGas,
            ),
            (
                format!("{op:#x}'s state-gas price"),
                SpecId::AMSTERDAM,
                create_code(op, 32, 0),
                HookHost::new(SpecId::AMSTERDAM).empty().price_fails(),
                AMPLE,
                InstructionResult::FatalExternalError,
            ),
        ]);
    }
    for (name, spec, code, mut host, gas_limit, expected) in cases {
        let (action, _) = run(spec, &code, gas_limit, 0, &mut host);
        assert_eq!(returned(action).0, expected, "{name}");
        assert!(!host.reached(), "{name}: the hook is not reached");
    }
}

/// With exactly the gas a start needs before its forward, it reaches the hook: the cases above fail
/// on the charge they name, not on one they share.
#[test]
fn test_a_start_with_the_gas_it_needs_reaches_the_hook() {
    let amsterdam = GasParams::new_spec(SpecId::AMSTERDAM);
    let call = call_pushes(CALL)
        + static_gas(SpecId::AMSTERDAM, CALL)
        + amsterdam.transfer_value_cost()
        + amsterdam.new_account_cost(true, true)
        + amsterdam.get(GasId::new_account_state_gas());
    let create = create_pushes(CREATE)
        + amsterdam.initcode_cost(32)
        + 3
        + amsterdam.create_cost()
        + amsterdam.get(GasId::create_state_gas());
    for (name, code, gas_limit) in [
        ("CALL", call_code(CALL, 1_000, 1, 0), call),
        ("CREATE", create_code(CREATE, 32, 0), create),
    ] {
        let mut host = HookHost::new(SpecId::AMSTERDAM).empty();
        let (action, _) = run(SpecId::AMSTERDAM, &code, gas_limit, 0, &mut host);

        let _ = started(action);
        assert_eq!(
            host.start().0.forward,
            0,
            "{name}: nothing is left to forward"
        );
    }
}
