//! The instruction hooks on the plain and the inspected path.
//!
//! `transact` runs a frame's instructions in the interpreter's loop and `inspect_tx` in the
//! inspector's; both run the same instructions, which call [`Host::write_completed`] and
//! [`Host::frame_starting`]. Each test runs a transaction both ways on [`HookContext`], which
//! delegates everything to the mainnet [`Context`] but the two hooks, and checks that the hooks are
//! reached at the same points with the same gas and that their answers end the transaction the same
//! way.

use context::{
    context_interface::{
        cfg::{GasId, GasParams, StateGasSite},
        host::{CompletedWrite, FrameStart, FrameStartKind, LoadError, WriteVerdict},
        journaled_state::AccountInfoLoad,
        result::{ExecutionResult, HaltReason, OutOfGasError, ResultAndState, ResultGas},
    },
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Host, Journal,
    LocalContext, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, ExecuteEvm, MainBuilder, MainContext,
    MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome, GasTracker,
    SStoreResult, SelfDestructResult, StateLoad,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, TxKind, B256, U256,
};
use revm_inspector::{InspectEvm, Inspector, NoOpInspector};
use state::{
    bytecode::opcode::{
        CALL, CALLCODE, CREATE, CREATE2, DELEGATECALL, LOG0, LOG1, MSTORE, POP, PUSH0, PUSH1,
        PUSH4, SELFDESTRUCT, SSTORE, STATICCALL, STOP,
    },
    AccountInfo, Bytecode,
};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as context::Database>::Error;

/// The account the transaction calls.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// The account [`PARENT`] starts frames on.
const CHILD: Address = address!("0x00000000000000000000000000000000000c41d0");
/// An account that does not exist, which `SELFDESTRUCT` pays.
const BENEFICIARY: Address = address!("0x00000000000000000000000000000000000be4e0");
/// An account that does not exist, which a call sends value to.
const EMPTY: Address = address!("0x00000000000000000000000000000000000e3970");
/// The identity precompile, a start the handler answers without a frame.
const IDENTITY: Address = address!("0x0000000000000000000000000000000000000004");
/// The transaction's gas limit.
const GAS_LIMIT: u64 = 5_000_000;
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
            other => panic!("a write this test does not know: {other:?}"),
        }
    }

    const fn address(&self) -> Address {
        match self {
            Self::Sstore(address, ..)
            | Self::Log(address, ..)
            | Self::SelfDestruct(address, ..) => *address,
        }
    }
}

/// What the hooks answer, and what they saw.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Hooks {
    verdict: WriteVerdict,
    admit: bool,
    /// History gas the first write hook charges on the gas it is handed.
    first_write_charge: u64,
    /// History gas the first start hook charges on the gas it is handed.
    first_start_charge: u64,
    /// Every write, with the gas as the hook found it.
    writes: Vec<(Write, GasTracker)>,
    /// Every start, with the gas as the hook found it.
    starts: Vec<(FrameStart, GasTracker)>,
}

impl Hooks {
    const fn observing() -> Self {
        Self {
            verdict: WriteVerdict::Continue,
            admit: true,
            first_write_charge: 0,
            first_start_charge: 0,
            writes: Vec::new(),
            starts: Vec::new(),
        }
    }
}

/// The mainnet [`Context`] with [`Hooks`].
struct HookContext {
    inner: MainnetContext<Db>,
    hooks: Hooks,
}

impl ContextTr for HookContext {
    type Block = BlockEnv;
    type Tx = TxEnv;
    type Cfg = CfgEnv;
    type Db = Db;
    type Journal = Journal<Db>;
    type Chain = ();
    type Local = LocalContext;

    fn all(
        &self,
    ) -> (
        &Self::Block,
        &Self::Tx,
        &Self::Cfg,
        &Self::Db,
        &Self::Journal,
        &Self::Chain,
        &Self::Local,
    ) {
        self.inner.all()
    }

    fn all_mut(
        &mut self,
    ) -> (
        &Self::Block,
        &Self::Tx,
        &Self::Cfg,
        &mut Self::Journal,
        &mut Self::Chain,
        &mut Self::Local,
    ) {
        self.inner.all_mut()
    }

    fn error(&mut self) -> &mut Result<(), ContextError<DbError>> {
        self.inner.error()
    }
}

impl ContextSetters for HookContext {
    fn set_tx(&mut self, tx: Self::Tx) {
        self.inner.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.inner.set_block(block);
    }
}

impl Host for HookContext {
    fn write_completed(&mut self, write: CompletedWrite<'_>, gas: &mut GasTracker) -> WriteVerdict {
        self.hooks.writes.push((Write::of(write), *gas));
        if self.hooks.writes.len() == 1 && self.hooks.first_write_charge > 0 {
            assert!(gas.record_history_cost(self.hooks.first_write_charge));
        }
        self.hooks.verdict.clone()
    }

    fn frame_starting(&mut self, start: &FrameStart, gas: &mut GasTracker) -> bool {
        self.hooks.starts.push((*start, *gas));
        if self.hooks.starts.len() == 1 && self.hooks.first_start_charge > 0 {
            assert!(gas.record_history_cost(self.hooks.first_start_charge));
        }
        self.hooks.admit
    }

    fn state_gas_price(&mut self, id: GasId, site: StateGasSite) -> Option<u64> {
        self.inner.state_gas_price(id, site)
    }

    fn basefee(&self) -> U256 {
        self.inner.basefee()
    }

    fn blob_gasprice(&self) -> U256 {
        self.inner.blob_gasprice()
    }

    fn gas_limit(&self) -> U256 {
        self.inner.gas_limit()
    }

    fn difficulty(&self) -> U256 {
        self.inner.difficulty()
    }

    fn prevrandao(&self) -> Option<U256> {
        self.inner.prevrandao()
    }

    fn block_number(&self) -> U256 {
        self.inner.block_number()
    }

    fn timestamp(&self) -> U256 {
        self.inner.timestamp()
    }

    fn beneficiary(&self) -> Address {
        self.inner.beneficiary()
    }

    fn slot_num(&self) -> U256 {
        self.inner.slot_num()
    }

    fn chain_id(&self) -> U256 {
        self.inner.chain_id()
    }

    fn effective_gas_price(&self) -> U256 {
        self.inner.effective_gas_price()
    }

    fn caller(&self) -> Address {
        self.inner.caller()
    }

    fn blob_hash(&self, number: usize) -> Option<U256> {
        self.inner.blob_hash(number)
    }

    fn max_initcode_size(&self) -> usize {
        self.inner.max_initcode_size()
    }

    fn gas_params(&self) -> &GasParams {
        self.inner.gas_params()
    }

    fn is_amsterdam_eip8037_enabled(&self) -> bool {
        self.inner.is_amsterdam_eip8037_enabled()
    }

    fn block_hash(&mut self, number: u64) -> Option<B256> {
        self.inner.block_hash(number)
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
        self.inner.selfdestruct(address, target, skip_cold_load)
    }

    fn log(&mut self, log: Log) {
        self.inner.log(log)
    }

    fn sstore_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError> {
        self.inner
            .sstore_skip_cold_load(address, key, value, skip_cold_load)
    }

    fn sload_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, LoadError> {
        self.inner
            .sload_skip_cold_load(address, key, skip_cold_load)
    }

    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue) {
        self.inner.tstore(address, key, value)
    }

    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue {
        self.inner.tload(address, key)
    }

    fn load_account_info_skip_cold_load(
        &mut self,
        address: Address,
        load_code: bool,
        skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError> {
        self.inner
            .load_account_info_skip_cold_load(address, load_code, skip_cold_load)
    }
}

/// The gas limit and reservoir of every frame the inspector saw start, the transaction's first.
#[derive(Default)]
struct Frames(Vec<(u64, u64)>);

impl Inspector<HookContext, EthInterpreter> for Frames {
    fn call(&mut self, _context: &mut HookContext, inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.0.push((inputs.gas_limit, inputs.reservoir));
        None
    }

    fn create(
        &mut self,
        _context: &mut HookContext,
        inputs: &mut CreateInputs,
    ) -> Option<CreateOutcome> {
        self.0.push((inputs.gas_limit(), inputs.reservoir()));
        None
    }
}

type HookEvm = Evm<
    HookContext,
    Frames,
    EthInstructions<EthInterpreter, HookContext>,
    EthPrecompiles,
    EthFrame<EthInterpreter>,
>;

/// A transaction to [`PARENT`] and the accounts it runs on.
struct Run {
    spec: SpecId,
    /// The transaction's reservoir: its cap is set this far below [`GAS_LIMIT`] on Amsterdam.
    reservoir: u64,
    /// `(address, code, balance)`.
    accounts: Vec<(Address, Vec<u8>, u64)>,
    hooks: Hooks,
}

impl Run {
    fn new(spec: SpecId, parent: Vec<u8>) -> Self {
        Self {
            spec,
            reservoir: 0,
            accounts: vec![(PARENT, parent, 10)],
            hooks: Hooks::observing(),
        }
    }

    fn with(mut self, address: Address, code: Vec<u8>, balance: u64) -> Self {
        self.accounts.push((address, code, balance));
        self
    }

    const fn reservoir(mut self, reservoir: u64) -> Self {
        self.reservoir = reservoir;
        self
    }

    fn hooks(mut self, hooks: Hooks) -> Self {
        self.hooks = hooks;
        self
    }

    fn context(&self) -> MainnetContext<Db> {
        let mut db = Db::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo::from_balance(U256::from(10u128.pow(21))),
        );
        for (address, code, balance) in &self.accounts {
            let info = AccountInfo::from_balance(U256::from(*balance))
                .with_nonce(1)
                .with_code(Bytecode::new_raw(Bytes::copy_from_slice(code)));
            db.insert_account_info(*address, info);
        }
        let (spec, reservoir) = (self.spec, self.reservoir);
        Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(spec);
            if spec.is_enabled_in(SpecId::AMSTERDAM) {
                cfg.tx_gas_limit_cap = Some(GAS_LIMIT - reservoir);
            }
        })
    }

    fn evm(&self) -> HookEvm {
        let ctx = HookContext {
            inner: self.context(),
            hooks: self.hooks.clone(),
        };
        Evm::new_with_inspector(
            ctx,
            Frames::default(),
            EthInstructions::new_mainnet_with_spec(self.spec),
            EthPrecompiles::new(self.spec),
        )
    }

    fn tx() -> TxEnv {
        TxEnv::builder()
            .caller(BENCH_CALLER)
            .kind(TxKind::Call(PARENT))
            .gas_price(0)
            .gas_limit(GAS_LIMIT)
            .build()
            .unwrap()
    }

    /// The transaction on the mainnet context, with the default hooks, with `transact` and with
    /// `inspect_tx`, which must end the same way.
    fn on_defaults(&self) -> ResultAndState {
        let transacted = self
            .context()
            .build_mainnet()
            .transact(Self::tx())
            .expect("the transaction runs");
        let inspected = self
            .context()
            .build_mainnet_with_inspector(NoOpInspector)
            .inspect_tx(Self::tx())
            .expect("the transaction runs");
        assert_eq!(inspected, transacted, "both paths end the same way");
        transacted
    }

    /// The transaction with `transact` and with `inspect_tx`, which must end the same way and
    /// reach the hooks at the same points with the same gas. Returns the outcome, what the hooks
    /// saw, and the frames the inspector saw start.
    fn on_both_paths(&self) -> (ResultAndState, Hooks, Vec<(u64, u64)>) {
        let mut plain = self.evm();
        let transacted = plain.transact(Self::tx()).expect("the transaction runs");
        let mut inspected = self.evm();
        let inspected_outcome = inspected
            .inspect_tx(Self::tx())
            .expect("the transaction runs");

        assert_eq!(inspected_outcome, transacted, "both paths end the same way");
        assert_eq!(
            inspected.ctx.hooks, plain.ctx.hooks,
            "both paths reach the hooks at the same points with the same gas"
        );
        (transacted, plain.ctx.hooks, inspected.inspector.0)
    }
}

/// `PUSHn` of `bytes`.
fn push(bytes: &[u8]) -> Vec<u8> {
    let mut code = vec![PUSH0 + bytes.len() as u8];
    code.extend_from_slice(bytes);
    code
}

/// The shortest push of `address`.
fn push_address(address: Address) -> Vec<u8> {
    let bytes = address.as_slice();
    let first = bytes.iter().position(|byte| *byte != 0).unwrap_or(19);
    push(&bytes[first..])
}

/// `SSTORE(0, 1)`.
fn sstore() -> Vec<u8> {
    vec![PUSH1, 1, PUSH0, SSTORE]
}

/// `LOG1` of 32 bytes of memory at offset 0.
fn log1() -> Vec<u8> {
    vec![PUSH1, 0xaa, PUSH1, 32, PUSH0, LOG1]
}

/// `SELFDESTRUCT` in favour of `target`.
fn selfdestruct(target: Address) -> Vec<u8> {
    let mut code = push_address(target);
    code.push(SELFDESTRUCT);
    code
}

/// `op` (a call) to `to` with `gas` and `value` (for `CALL` and `CALLCODE`), no input and no
/// output, dropping the success flag.
fn call(op: u8, to: Address, gas: u32, value: u8) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0];
    if matches!(op, CALL | CALLCODE) {
        code.extend_from_slice(&[PUSH1, value]);
    }
    code.extend(push_address(to));
    code.push(PUSH4);
    code.extend_from_slice(&gas.to_be_bytes());
    code.extend_from_slice(&[op, POP]);
    code
}

/// `op` (`CREATE` or `CREATE2`) of `initcode`, at most 32 bytes, endowed with nothing, dropping
/// the address.
fn create(op: u8, initcode: &[u8]) -> Vec<u8> {
    let len = initcode.len() as u8;
    let mut code = push(initcode);
    code.extend_from_slice(&[PUSH0, MSTORE]);
    if op == CREATE2 {
        code.extend_from_slice(&[PUSH1, 0x5a]);
    }
    code.extend_from_slice(&[PUSH1, len, PUSH1, 32 - len, PUSH0, op, POP]);
    code
}

/// Code that starts a frame of its own: a `STATICCALL` of [`IDENTITY`], which a static frame may
/// make too.
fn nested_start() -> Vec<u8> {
    let mut code = call(STATICCALL, IDENTITY, 0, 0);
    code.push(STOP);
    code
}

fn ends(mut code: Vec<u8>) -> Vec<u8> {
    code.push(STOP);
    code
}

/// A frame of each kind [`PARENT`] starts: on [`CHILD`], whose code starts a frame of its own, or
/// with that code as init code.
fn start_cases() -> Vec<(FrameStartKind, Vec<u8>)> {
    vec![
        (FrameStartKind::Call, ends(call(CALL, CHILD, 100_000, 1))),
        (
            FrameStartKind::CallCode,
            ends(call(CALLCODE, CHILD, 100_000, 1)),
        ),
        (
            FrameStartKind::DelegateCall,
            ends(call(DELEGATECALL, CHILD, 100_000, 0)),
        ),
        (
            FrameStartKind::StaticCall,
            ends(call(STATICCALL, CHILD, 100_000, 0)),
        ),
        (
            FrameStartKind::Create,
            ends(create(CREATE, &nested_start())),
        ),
        (
            FrameStartKind::Create2,
            ends(create(CREATE2, &nested_start())),
        ),
    ]
}

/// [`PARENT`] writes, logs, starts a frame of each kind and self-destructs; [`CHILD`] logs.
fn every_site() -> Run {
    let mut parent = sstore();
    parent.extend(log1());
    parent.extend(call(CALL, CHILD, 50_000, 1));
    parent.extend(call(CALLCODE, CHILD, 50_000, 1));
    parent.extend(call(DELEGATECALL, CHILD, 50_000, 0));
    parent.extend(call(STATICCALL, CHILD, 50_000, 0));
    parent.extend(create(CREATE, &[]));
    parent.extend(create(CREATE2, &[]));
    parent.extend(selfdestruct(BENEFICIARY));
    Run::new(SpecId::PRAGUE, parent).with(CHILD, vec![PUSH0, PUSH0, LOG0, STOP], 0)
}

/// [`every_site`]'s gas and number of logs as the fork reported them before the hooks, on a spec
/// before EIP-8037 and on one with it.
const EVERY_SITE: [(SpecId, ResultGas, usize); 2] = [
    (
        SpecId::PRAGUE,
        ResultGas::new_with_state_gas(208_269, 0, 21_000, 0),
        4,
    ),
    // Two more logs: EIP-7708's transfer logs for the value `CALL` and the `SELFDESTRUCT`.
    (
        SpecId::AMSTERDAM,
        ResultGas::new_with_state_gas(790_389, 0, 15_000, 648_720),
        6,
    ),
];

/// The default hooks change nothing: on a spec before EIP-8037 and on one with it, a transaction
/// that reaches every site reports what it reported before the hooks, and hooks that only observe
/// change nothing on either path. The observing hooks see every site.
#[test]
fn test_hooks_that_only_observe_change_nothing() {
    for (spec, gas, logs) in EVERY_SITE {
        let run = Run {
            spec,
            ..every_site()
        };
        let defaults = run.on_defaults();
        let (outcome, hooks, _) = run.on_both_paths();

        assert_eq!(outcome, defaults, "{spec:?}");
        assert!(
            outcome.result.is_success(),
            "{spec:?}: {:?}",
            outcome.result
        );
        assert_eq!(*outcome.result.gas(), gas, "{spec:?}");
        assert_eq!(outcome.result.logs().len(), logs, "{spec:?}");

        let writes: Vec<Write> = hooks.writes.into_iter().map(|(write, _)| write).collect();
        let set = SStoreResult {
            original_value: StorageValue::ZERO,
            present_value: StorageValue::ZERO,
            new_value: StorageValue::from(1),
        };
        let destructed = SelfDestructResult {
            had_value: true,
            target_exists: false,
            previously_destroyed: false,
        };
        assert_eq!(
            writes,
            vec![
                Write::Sstore(PARENT, StorageKey::ZERO, set),
                Write::Log(PARENT, 1, 32),
                Write::Log(CHILD, 0, 0),
                Write::Log(PARENT, 0, 0),
                Write::Log(PARENT, 0, 0),
                Write::SelfDestruct(PARENT, BENEFICIARY, destructed),
            ],
            "{spec:?}: the static call's log fails before the hook"
        );
        let kinds: Vec<FrameStartKind> = hooks.starts.iter().map(|(start, _)| start.kind).collect();
        assert_eq!(
            kinds,
            [
                FrameStartKind::Call,
                FrameStartKind::CallCode,
                FrameStartKind::DelegateCall,
                FrameStartKind::StaticCall,
                FrameStartKind::Create,
                FrameStartKind::Create2,
            ],
            "{spec:?}"
        );
    }
}

/// `code` with its last instruction, which drops a result, replaced by `STOP`.
fn stops_after(mut code: Vec<u8>) -> Vec<u8> {
    assert_eq!(code.pop(), Some(POP));
    ends(code)
}

/// The instruction under test, on [`CHILD`], which [`PARENT`] calls with `gas`: the first is its
/// name, the second its code, which ends with the instruction. `CHILD` has a balance of 1, to send
/// or pay out; the calls go to an account that does not exist, so it is cold.
fn charge_cases() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("SSTORE", ends(sstore())),
        ("LOG1", ends(log1())),
        ("SELFDESTRUCT", selfdestruct(BENEFICIARY)),
        ("CALL", stops_after(call(CALL, EMPTY, 0, 1))),
        ("CALLCODE", stops_after(call(CALLCODE, EMPTY, 0, 1))),
        ("DELEGATECALL", stops_after(call(DELEGATECALL, EMPTY, 0, 0))),
        ("STATICCALL", stops_after(call(STATICCALL, EMPTY, 0, 0))),
        ("CREATE", stops_after(create(CREATE, &[]))),
        ("CREATE2", stops_after(create(CREATE2, &[]))),
    ]
}

/// On each path, an instruction one gas short of what it charges before its hook fails without
/// reaching it, and with exactly that gas reaches it with nothing left and nothing is charged
/// after it: the frame, which ends with the instruction, succeeds. On Amsterdam with no reservoir,
/// so the last charge is the state-gas charge where there is one.
#[test]
fn test_a_charge_that_fails_before_the_hook_never_reaches_it() {
    // The parent stores whether the child succeeded in its slot 0.
    let mut parent = stops_after(call(CALL, CHILD, 0, 0));
    parent.pop();
    parent.extend_from_slice(&[PUSH0, SSTORE, STOP]);
    // The named gas: the four bytes before `CALL PUSH0 SSTORE STOP`.
    let gas_at = parent.len() - 8;
    let child_run = |code: &[u8], gas: u32| {
        let mut parent = parent.clone();
        parent[gas_at..gas_at + 4].copy_from_slice(&gas.to_be_bytes());
        let (outcome, hooks, _) = Run::new(SpecId::AMSTERDAM, parent)
            .with(CHILD, code.to_vec(), 1)
            .on_both_paths();
        let succeeded = outcome.state[&PARENT].storage[&StorageKey::ZERO].present_value;
        (hooks, succeeded == U256::from(1))
    };
    // The child's own hook call: its write, or the start it makes after the parent's.
    let childs = |hooks: &Hooks| -> Option<(u64, u64)> {
        if let Some((_, gas)) = hooks
            .writes
            .iter()
            .find(|(write, _)| write.address() == CHILD)
        {
            return Some((gas.remaining(), 0));
        }
        hooks
            .starts
            .get(1)
            .map(|(start, gas)| (gas.remaining(), start.forward))
    };
    for (name, code) in charge_cases() {
        let ample: u32 = 1_000_000;
        let (hooks, succeeded) = child_run(&code, ample);
        let (left, forward) = childs(&hooks).expect("the hook is reached");
        let needed = u64::from(ample) - left - forward;
        assert!(succeeded, "{name}");

        let (hooks, succeeded) = child_run(&code, needed as u32);
        assert_eq!(
            childs(&hooks),
            Some((0, 0)),
            "{name}: reached with nothing left"
        );
        assert!(succeeded, "{name}: nothing is charged after the hook");

        let (hooks, succeeded) = child_run(&code, needed as u32 - 1);
        assert_eq!(childs(&hooks), None, "{name}: one gas short is not reached");
        assert!(!succeeded, "{name}");
    }
}

/// [`PARENT`] runs `code` and stops, with what it changes: slot 0, a log, or [`BENEFICIARY`]'s
/// balance.
fn verdict_cases() -> Vec<(&'static str, Vec<u8>)> {
    vec![
        ("SSTORE", ends(sstore())),
        ("LOG1", ends(log1())),
        ("SELFDESTRUCT", selfdestruct(BENEFICIARY)),
    ]
}

/// What the transaction left of the write: slot 0's value, the logs, and what [`BENEFICIARY`] got.
fn written(outcome: &ResultAndState) -> (U256, usize, U256) {
    let slot = outcome
        .state
        .get(&PARENT)
        .and_then(|account| account.storage.get(&StorageKey::ZERO))
        .map_or(U256::ZERO, |slot| slot.present_value);
    let paid = outcome
        .state
        .get(&BENEFICIARY)
        .map_or(U256::ZERO, |account| account.info.balance);
    (slot, outcome.result.logs().len(), paid)
}

/// Each verdict ends the transaction the same way on both paths: `Continue` keeps the write,
/// `OutOfGas` halts and `Revert` reverts with the verdict's output, on the gas the hook left, and
/// both undo it; a `SELFDESTRUCT` so stopped pays nothing.
#[test]
fn test_each_verdict_ends_the_write_the_same_way_on_both_paths() {
    let kept = [
        (U256::from(1), 0, U256::ZERO),
        (U256::ZERO, 1, U256::ZERO),
        (U256::ZERO, 0, U256::from(10)),
    ];
    for ((name, code), kept) in verdict_cases().into_iter().zip(kept) {
        let run = |verdict| {
            Run::new(SpecId::OSAKA, code.clone())
                .hooks(Hooks {
                    verdict,
                    ..Hooks::observing()
                })
                .on_both_paths()
        };

        let (outcome, _, _) = run(WriteVerdict::Continue);
        assert!(outcome.result.is_success(), "{name}: {:?}", outcome.result);
        assert_eq!(written(&outcome), kept, "{name}: the write is kept");

        let (outcome, _, _) = run(WriteVerdict::OutOfGas);
        let ExecutionResult::Halt { reason, gas, .. } = &outcome.result else {
            panic!("{name}: {:?}", outcome.result);
        };
        assert_eq!(*reason, HaltReason::OutOfGas(OutOfGasError::Basic));
        assert_eq!(gas.total_gas_spent(), GAS_LIMIT, "{name}");
        assert_eq!(written(&outcome), (U256::ZERO, 0, U256::ZERO), "{name}");

        let (outcome, hooks, _) = run(WriteVerdict::Revert(Bytes::from_static(STOP_OUTPUT)));
        let ExecutionResult::Revert { output, gas, .. } = &outcome.result else {
            panic!("{name}: {:?}", outcome.result);
        };
        assert_eq!(*output, Bytes::from_static(STOP_OUTPUT), "{name}");
        assert_eq!(
            gas.total_gas_spent(),
            GAS_LIMIT - hooks.writes[0].1.remaining(),
            "{name}: the frame ends on the gas the hook left"
        );
        assert_eq!(written(&outcome), (U256::ZERO, 0, U256::ZERO), "{name}");
    }
}

/// `false` fails the starting instruction out of gas and starts no frame, on both paths: the
/// frame's code never runs, so its own start never reaches the hook, and the inspector sees no
/// frame start.
#[test]
fn test_a_refused_start_starts_no_frame_on_both_paths() {
    for (kind, parent) in start_cases() {
        let run = Run::new(SpecId::OSAKA, parent).with(CHILD, nested_start(), 10);

        let (admitted, hooks, frames) = run.on_both_paths();
        assert!(admitted.result.is_success(), "{kind:?}");
        assert_eq!(hooks.starts.len(), 2, "{kind:?}: the frame ran");
        assert_eq!(frames.len(), 3, "{kind:?}");

        let (refused, hooks, frames) = run
            .hooks(Hooks {
                admit: false,
                ..Hooks::observing()
            })
            .on_both_paths();
        let ExecutionResult::Halt { reason, .. } = &refused.result else {
            panic!("{kind:?}: {:?}", refused.result);
        };
        assert_eq!(
            *reason,
            HaltReason::OutOfGas(OutOfGasError::Basic),
            "{kind:?}"
        );
        assert_eq!(hooks.starts.len(), 1, "{kind:?}: the frame never ran");
        assert_eq!(hooks.starts[0].0.kind, kind);
        assert_eq!(frames.len(), 1, "{kind:?}: no frame started but the first");
    }
}

/// A started frame inherits the reservoir as the hook left it, on both paths: the parent's hook
/// charges history gas the reservoir covers, and the frame finds the rest, whether it runs its own
/// code or is answered without a frame, as a precompile is.
#[test]
fn test_a_started_frame_inherits_the_reservoir_the_hook_left() {
    const CHARGE: u64 = 4_000;
    let hooks = Hooks {
        first_start_charge: CHARGE,
        ..Hooks::observing()
    };
    for (kind, parent) in start_cases() {
        let (_, hooks, frames) = Run::new(SpecId::AMSTERDAM, parent)
            .with(CHILD, nested_start(), 10)
            .reservoir(1_000_000)
            .hooks(hooks.clone())
            .on_both_paths();
        let handed = hooks.starts[0].1.reservoir();
        let inherited = hooks.starts[1].1.reservoir();

        assert!(handed > CHARGE, "{kind:?}: the reservoir covers the charge");
        assert_eq!(inherited, handed - CHARGE, "{kind:?}");
        assert_eq!(frames[1].1, handed - CHARGE, "{kind:?}: the frame's input");
    }

    // The parent calls the identity precompile, then logs: the reservoir the answered call returns
    // is the one its input carried.
    let mut parent = call(STATICCALL, IDENTITY, 100_000, 0);
    parent.extend_from_slice(&[PUSH0, PUSH0, LOG0, STOP]);
    let (_, hooks, frames) = Run::new(SpecId::AMSTERDAM, parent)
        .reservoir(1_000_000)
        .hooks(hooks)
        .on_both_paths();
    let handed = hooks.starts[0].1.reservoir();

    assert_eq!(hooks.writes[0].1.reservoir(), handed - CHARGE);
    assert_eq!(frames[1].1, handed - CHARGE);
}

/// On both paths, a call's forward is the gas it names, or 63/64 of what the caller has left when it
/// names more, and a creation's is always the cap; the hook sees it already taken; and the frame's
/// gas limit is the forward with a value call's stipend on top.
#[test]
fn test_the_forward_on_both_paths() {
    let stipend = GasParams::new_spec(SpecId::OSAKA).call_stipend();
    let run = |parent: Vec<u8>| {
        let (_, hooks, frames) = Run::new(SpecId::OSAKA, parent)
            .with(CHILD, vec![STOP], 0)
            .on_both_paths();
        (hooks.starts[0], frames[1].0)
    };
    for (op, value) in [(CALL, 1), (CALLCODE, 1), (DELEGATECALL, 0), (STATICCALL, 0)] {
        let ((named, named_gas), named_limit) = run(ends(call(op, CHILD, 30_000, value)));
        let ((capped, capped_gas), capped_limit) = run(ends(call(op, CHILD, u32::MAX, value)));
        // Both runs had the same gas before the forward.
        let left = capped_gas.remaining() + capped.forward;
        let stipend = if value > 0 { stipend } else { 0 };

        assert_eq!(named.forward, 30_000, "{op:#x}");
        assert_eq!(capped.forward, left - left / 64, "{op:#x}: the 63/64 cap");
        assert_eq!(
            named_gas.remaining() + named.forward,
            left,
            "{op:#x}: the hook sees the forward taken"
        );
        assert_eq!(named_limit, named.forward + stipend, "{op:#x}");
        assert_eq!(capped_limit, capped.forward + stipend, "{op:#x}");
    }
    for op in [CREATE, CREATE2] {
        let ((start, gas), limit) = run(ends(create(op, &[])));
        let left = gas.remaining() + start.forward;

        assert_eq!(
            start.forward,
            left - left / 64,
            "{op:#x}: taken, and the 63/64 cap"
        );
        assert_eq!(limit, start.forward, "{op:#x}");
    }
}

/// On both paths, what a hook charges stands and the instruction goes on with the gas as the hook
/// left it: a charge the reservoir covers comes out of the reservoir, and one that spills empties
/// it and takes the rest from regular gas. A start's forward is what was taken before the charge.
#[test]
fn test_a_hook_charge_stands_on_both_paths() {
    const RESERVOIR: u64 = 100_000;

    // The first `LOG1`'s hook charges; the second sees what it left.
    let mut parent = log1();
    parent.extend(log1());
    let run = |charge| {
        Run::new(SpecId::AMSTERDAM, ends(parent.clone()))
            .reservoir(RESERVOIR)
            .hooks(Hooks {
                first_write_charge: charge,
                ..Hooks::observing()
            })
            .on_both_paths()
            .1
    };
    let quiet = run(0).writes[1].1;
    let handed = quiet.reservoir();
    for (charge, reservoir, spilled) in [(4_000, handed - 4_000, 0), (handed + 500, 0, 500)] {
        let second = run(charge).writes[1].1;

        assert_eq!(second.reservoir(), reservoir, "charge {charge}");
        assert_eq!(
            second.remaining(),
            quiet.remaining() - spilled,
            "charge {charge}"
        );
        assert_eq!(second.history_gas_spent(), charge as i64, "charge {charge}");
    }

    // The `CALL`'s hook charges; the frame inherits the reservoir it left, with the gas limit it
    // would have had, and the parent logs once the frame returns.
    let mut parent = call(CALL, CHILD, 100_000, 1);
    parent.extend_from_slice(&[PUSH0, PUSH0, LOG0, STOP]);
    let run = |charge| {
        Run::new(SpecId::AMSTERDAM, parent.clone())
            .with(CHILD, nested_start(), 0)
            .reservoir(RESERVOIR)
            .hooks(Hooks {
                first_start_charge: charge,
                ..Hooks::observing()
            })
            .on_both_paths()
    };
    let (_, quiet, quiet_frames) = run(0);
    let handed = quiet.starts[0].1.reservoir();
    for (charge, reservoir, spilled) in [(4_000, handed - 4_000, 0), (handed + 500, 0, 500)] {
        let (_, hooks, frames) = run(charge);

        assert_eq!(
            hooks.starts[1].1.reservoir(),
            reservoir,
            "charge {charge}: inherited"
        );
        assert_eq!(
            frames[1],
            (quiet_frames[1].0, reservoir),
            "charge {charge}: the input"
        );
        let (after, quiet_after) = (hooks.writes[0].1, quiet.writes[0].1);
        assert_eq!(after.reservoir(), reservoir, "charge {charge}: returned");
        assert_eq!(
            after.remaining(),
            quiet_after.remaining() - spilled,
            "charge {charge}"
        );
    }
}
