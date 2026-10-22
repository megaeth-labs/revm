//! The byte pool, the start charge and the history allowance at the call and creation
//! instructions.
//!
//! Once [`Host::frame_starting`] has answered, the instruction copies the start charge the hook
//! declared on the start onto the frame's input, takes the child's share of the caller's byte pool
//! out of the pool and puts it on the input too, and, for a value call, gives the input the
//! schedule's history allowance. Each test runs one frame on [`PoolHost`], whose start hook draws
//! bytes from the gas it is handed, declares a start charge, and admits or refuses the start as
//! the test tells it.

use bytecode::{opcode::*, Bytecode};
use context_interface::{
    cfg::{GasId, GasParams, StateGasSite},
    context::{SStoreResult, SelfDestructResult, StateLoad},
    host::{CompletedWrite, FrameStart, FrameStartKind, LoadError, WriteVerdict},
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
    FrameInput, Gas, GasTracker, Host, InputsImpl, InstructionResult, Interpreter,
    InterpreterAction, PoolAmounts, PoolId, SharedMemory, StartCharge,
};
use state::AccountInfo;
use std::borrow::Cow;

/// The account the frame runs as.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// The frame's caller.
const CALLER: Address = address!("0x00000000000000000000000000000000000ca11e");
/// The account a call names.
const TARGET: Address = address!("0x0000000000000000000000000000000000007a59");
/// Gas enough for every frame here.
const AMPLE: u64 = 1_000_000;
/// The pool the frame starts with, unless a test leaves it unlimited.
const POOL: u64 = 1_000;
/// The pool the tests draw: any of the tracker's pools behaves the same.
const FIRST: PoolId = PoolId::new(0);
/// A start charge a hook declares.
const DECLARED: StartCharge = StartCharge::new(3_520, PoolAmounts::ZERO.with(FIRST, 160));
/// What a chain's schedule sets: the child's share and a value call's allowance.
const PERCENT: u64 = 98;
const ALLOWANCE: u64 = 160;

/// A [`DummyHost`] whose start hook draws [`PoolHost::draw`] bytes from the gas it is handed,
/// declares [`PoolHost::declare`] on the start, records what it saw, and answers
/// [`PoolHost::admit`].
struct PoolHost {
    dummy: DummyHost,
    spec: SpecId,
    params: GasParams,
    draw: u64,
    declare: StartCharge,
    admit: bool,
    /// Every start the hook saw and the gas, both as the hook found them, and whether its draw
    /// paid.
    starts: Vec<(FrameStart, GasTracker, bool)>,
}

impl PoolHost {
    fn new(spec: SpecId) -> Self {
        Self {
            dummy: DummyHost::new(spec),
            spec,
            params: GasParams::new_spec(spec),
            draw: 0,
            declare: StartCharge::default(),
            admit: true,
            starts: Vec::new(),
        }
    }

    /// The schedule of a chain that keeps part of the pool for the caller and gives a value
    /// call's child an allowance.
    fn on_a_chains_schedule(mut self) -> Self {
        self.params.override_gas([
            (GasId::pool_forward_percent(FIRST), PERCENT),
            (GasId::call_history_allowance(), ALLOWANCE),
        ]);
        self
    }

    const fn drawing(mut self, draw: u64) -> Self {
        self.draw = draw;
        self
    }

    const fn declaring(mut self, charge: StartCharge) -> Self {
        self.declare = charge;
        self
    }

    const fn refusing(mut self) -> Self {
        self.admit = false;
        self
    }

    /// The one start the hook saw.
    fn start(&self) -> &(FrameStart, GasTracker, bool) {
        assert_eq!(self.starts.len(), 1, "one start: {:?}", self.starts);
        &self.starts[0]
    }
}

impl Host for PoolHost {
    fn write_completed(&mut self, _: CompletedWrite<'_>, _: &mut GasTracker) -> WriteVerdict {
        WriteVerdict::Continue
    }

    fn frame_starting(&mut self, start: &mut FrameStart, gas: &mut GasTracker) -> bool {
        let found = (*start, *gas);
        let paid = gas.record_pool(FIRST, self.draw);
        start.charge = self.declare;
        self.starts.push((found.0, found.1, paid));
        self.admit && paid
    }

    fn state_gas_price(&mut self, id: GasId, _site: StateGasSite) -> Option<u64> {
        Some(self.gas_params().get(id))
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
        &self.params
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
        _skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
        Ok(StateLoad::new(SelfDestructResult::default(), false))
    }

    fn log(&mut self, log: Log) {
        self.dummy.log(log)
    }

    fn sstore_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError> {
        self.dummy
            .sstore_skip_cold_load(address, key, value, skip_cold_load)
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
        _skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError> {
        Ok(AccountInfoLoad {
            account: Cow::Owned(AccountInfo::from_balance(U256::from(AMPLE))),
            is_cold: false,
            is_empty: false,
        })
    }
}

/// Runs `code` as [`CONTRACT`], called by [`CALLER`], with [`AMPLE`] gas and a byte pool of
/// `pool`, if given, and returns what the frame asked for and its gas.
fn run(code: &[u8], pool: Option<u64>, host: &mut PoolHost) -> (InterpreterAction, Gas) {
    let spec = host.spec;
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::copy_from_slice(code))),
        InputsImpl {
            target_address: CONTRACT,
            caller_address: CALLER,
            ..Default::default()
        },
        false,
        spec,
        AMPLE,
    );
    if let Some(pool) = pool {
        interpreter.gas.set_pool_remaining(FIRST, pool);
    }
    let table = instruction_table::<EthInterpreter, PoolHost>();
    let action = interpreter.run_plain(&table, &gas_table_spec(spec), host);
    (action, interpreter.gas)
}

/// The frame a frame asked to start.
fn started(action: InterpreterAction) -> FrameInput {
    match action {
        InterpreterAction::NewFrame(input) => input,
        other => panic!("expected a new frame, got {other:?}"),
    }
}

/// The result a frame returned with.
fn returned(action: InterpreterAction) -> InstructionResult {
    match action {
        InterpreterAction::Return(result) => result.result,
        other => panic!("expected the frame to return, got {other:?}"),
    }
}

/// The pool figures of a started frame's input: its share, its start charge and its history
/// allowance (a creation has none).
fn figures(input: &FrameInput) -> (u64, StartCharge, u64) {
    match input {
        FrameInput::Call(inputs) => (
            inputs.pool_shares.get(FIRST),
            inputs.charged_start,
            inputs.history_allowance,
        ),
        FrameInput::Create(inputs) => (inputs.pool_shares().get(FIRST), inputs.charged_start(), 0),
        FrameInput::Empty => unreachable!(),
    }
}

/// `op` (a call) to [`TARGET`] with `value` (for `CALL` and `CALLCODE`), no input and no output,
/// then `STOP`.
fn call_code(op: u8, value: u8) -> Vec<u8> {
    let mut code = vec![PUSH1, 0, PUSH1, 0, PUSH1, 0, PUSH1, 0];
    if matches!(op, CALL | CALLCODE) {
        code.extend_from_slice(&[PUSH1, value]);
    }
    code.push(PUSH20);
    code.extend_from_slice(TARGET.as_slice());
    code.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, op, STOP]);
    code
}

/// `op` (`CREATE` or `CREATE2`) of one byte of memory, endowed with nothing, then `STOP`.
fn create_code(op: u8) -> Vec<u8> {
    let mut code = Vec::new();
    if op == CREATE2 {
        code.extend_from_slice(&[PUSH1, 0x5a]);
    }
    code.extend_from_slice(&[PUSH1, 1, PUSH1, 0, PUSH1, 0, op, STOP]);
    code
}

/// A start of each kind.
fn start_cases() -> Vec<(FrameStartKind, Vec<u8>)> {
    vec![
        (FrameStartKind::Call, call_code(CALL, 1)),
        (FrameStartKind::CallCode, call_code(CALLCODE, 1)),
        (FrameStartKind::DelegateCall, call_code(DELEGATECALL, 0)),
        (FrameStartKind::StaticCall, call_code(STATICCALL, 0)),
        (FrameStartKind::Create, create_code(CREATE)),
        (FrameStartKind::Create2, create_code(CREATE2)),
    ]
}

/// With the schedule as it is, every start forwards the whole pool: the child's share is the
/// caller's pool, the caller keeps nothing, and the forward is not a draw. The default hook
/// declares no start charge, and no frame gets an allowance.
#[test]
fn test_every_start_forwards_the_whole_pool_by_default() {
    for (kind, code) in start_cases() {
        let mut host = PoolHost::new(SpecId::OSAKA);
        let (action, gas) = run(&code, Some(POOL), &mut host);
        let input = started(action);

        assert_eq!(host.start().0.kind, kind);
        assert_eq!(
            figures(&input),
            (POOL, StartCharge::default(), 0),
            "{kind:?}"
        );
        assert_eq!(
            gas.pool_remaining(FIRST),
            0,
            "{kind:?}: the caller keeps nothing"
        );
        assert_eq!(
            gas.pool_spent(FIRST),
            0,
            "{kind:?}: a forward is not a draw"
        );
        assert!(!gas.pool_crossing(FIRST), "{kind:?}");
    }
}

/// A frame that was given no pool has an unlimited one, and forwards it unlimited.
#[test]
fn test_an_unlimited_pool_is_forwarded_unlimited() {
    for (kind, code) in start_cases() {
        let mut host = PoolHost::new(SpecId::OSAKA);
        let (action, gas) = run(&code, None, &mut host);
        let (share, _, _) = figures(&started(action));

        assert_eq!(share, u64::MAX, "{kind:?}");
        assert_eq!(gas.pool_remaining(FIRST), 0, "{kind:?}");
    }
}

/// The share is taken from the pool as the hook left it, by the schedule's percentage, rounded
/// down; the caller keeps the rest, and the hook's draw stays drawn.
#[test]
fn test_the_share_follows_the_schedule_after_the_hooks_draw() {
    for (kind, code) in start_cases() {
        let mut host = PoolHost::new(SpecId::OSAKA)
            .on_a_chains_schedule()
            .drawing(200);
        let (action, gas) = run(&code, Some(POOL), &mut host);
        let (share, _, _) = figures(&started(action));

        let (_, found, paid) = host.start();
        assert_eq!(
            found.pool_remaining(FIRST),
            POOL,
            "{kind:?}: the hook found the pool"
        );
        assert!(paid);
        // 800 left after the draw; 98% of it, rounded down.
        assert_eq!(share, 784, "{kind:?}");
        assert_eq!(
            gas.pool_remaining(FIRST),
            16,
            "{kind:?}: the caller keeps the rest"
        );
        assert_eq!(
            gas.pool_spent(FIRST),
            200,
            "{kind:?}: the hook's draw stays"
        );
    }
}

/// A start reaches the hook with nothing declared, and the start charge the hook declares on it
/// rides on the frame's input.
#[test]
fn test_the_declared_start_charge_rides_on_the_input() {
    for (kind, code) in start_cases() {
        let mut host = PoolHost::new(SpecId::OSAKA).declaring(DECLARED);
        let (action, _) = run(&code, Some(POOL), &mut host);
        let (_, charged, _) = figures(&started(action));

        assert!(host.start().0.charge.is_zero(), "{kind:?}: none at first");
        assert_eq!(charged, DECLARED, "{kind:?}");
    }
}

/// A start the hook refuses fails out of gas and starts no frame: the hook's draw stays, as the
/// forward of regular gas does, no share is taken, and the declared charge goes with the start.
#[test]
fn test_a_refused_start_forwards_no_bytes_and_discards_the_declared_charge() {
    for (kind, code) in start_cases() {
        let mut host = PoolHost::new(SpecId::OSAKA)
            .drawing(200)
            .declaring(DECLARED)
            .refusing();
        let (action, gas) = run(&code, Some(POOL), &mut host);

        assert_eq!(returned(action), InstructionResult::OutOfGas, "{kind:?}");
        assert_eq!(
            gas.pool_remaining(FIRST),
            POOL - 200,
            "{kind:?}: no share taken"
        );
        assert_eq!(gas.pool_spent(FIRST), 200, "{kind:?}");
        assert!(!gas.pool_crossing(FIRST), "{kind:?}");
    }
}

/// A draw the pool cannot pay takes nothing and records the crossing; the hook turns it into a
/// refusal, and the record, with the pool as it was, is readable after the halt zeroed the gas.
#[test]
fn test_a_draw_the_pool_cannot_pay_records_a_crossing_that_survives_the_halt() {
    for (kind, code) in start_cases() {
        let mut host = PoolHost::new(SpecId::OSAKA).drawing(POOL + 1);
        let (action, gas) = run(&code, Some(POOL), &mut host);

        assert_eq!(returned(action), InstructionResult::OutOfGas, "{kind:?}");
        assert!(!host.start().2, "{kind:?}: the draw failed");
        assert_eq!(
            gas.remaining(),
            0,
            "{kind:?}: the halt zeroed the regular gas"
        );
        assert_eq!(
            gas.pool_remaining(FIRST),
            POOL,
            "{kind:?}: the failed draw took nothing"
        );
        assert_eq!(gas.pool_spent(FIRST), 0, "{kind:?}");
        assert!(gas.pool_crossing(FIRST), "{kind:?}");
    }
}

/// A value call's child gets the schedule's history allowance where it gets the call stipend;
/// no other start does, and the default schedule gives none.
#[test]
fn test_the_history_allowance_is_set_on_a_value_call_only() {
    let cases = [
        ("CALL with value", call_code(CALL, 1), ALLOWANCE),
        ("CALLCODE with value", call_code(CALLCODE, 1), ALLOWANCE),
        ("CALL without value", call_code(CALL, 0), 0),
        ("CALLCODE without value", call_code(CALLCODE, 0), 0),
        ("DELEGATECALL", call_code(DELEGATECALL, 0), 0),
        ("STATICCALL", call_code(STATICCALL, 0), 0),
    ];
    for (name, code, allowance) in &cases {
        let mut host = PoolHost::new(SpecId::OSAKA).on_a_chains_schedule();
        let (action, _) = run(code, Some(POOL), &mut host);
        let (_, _, found) = figures(&started(action));
        assert_eq!(found, *allowance, "{name}");

        let mut host = PoolHost::new(SpecId::OSAKA);
        let (action, _) = run(code, Some(POOL), &mut host);
        let (_, _, found) = figures(&started(action));
        assert_eq!(found, 0, "{name}: none on the default schedule");
    }
}
