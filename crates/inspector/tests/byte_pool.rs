//! The byte pool, the start charge and the history allowance on the plain and the inspected path.
//!
//! `transact` runs the frames in the handler's loop and `inspect_tx` in the inspector's; both
//! build frames with `make_call_frame` and `make_create_frame` and settle them with
//! `return_result`, so a child's share, its start charge and its allowance travel the same way
//! and settle to the same figures. Each test runs a transaction both ways on the mainnet context,
//! with a start hook of the test's own ([`DeclaringContext`]) and instructions of the test's own,
//! which stand in for a consumer's hooks, and checks that the two paths end the same way; an
//! inspector records each frame's input as it starts and its outcome as it ends, before its caller
//! settles it, and the figures are checked on the inspected path (the handler's tests check them
//! on the plain one). Two tests run on the inspected path alone: an outcome the inspector builds
//! or rewrites itself has no counterpart on the plain path.

use context::{
    context_interface::{
        cfg::{GasId, GasParams, StateGasSite},
        host::{FrameStart, LoadError},
        journaled_state::AccountInfoLoad,
    },
    result::ResultAndState,
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Host, Journal,
    LocalContext, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, ExecuteEvm, MainContext,
    MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome, Gas,
    GasTracker, Instruction, InstructionContext, InstructionExecResult, InstructionResult,
    InterpreterResult, SStoreResult, SelfDestructResult, StartCharge, StateLoad,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, TxKind, B256, U256,
};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{
        CALL, CREATE, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3, REVERT, STOP,
    },
    AccountInfo, Bytecode,
};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as context::Database>::Error;
type Ctx = DeclaringContext;
type PoolEvm = Evm<Ctx, Figures, EthInstructions<EthInterpreter, Ctx>, EthPrecompiles, EthFrame>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 3_000_000;
/// What `DRAW` draws.
const DRAW_BYTES: u64 = 100;
/// What the start hook charges and declares after `DECLARE`.
const DECLARED: StartCharge = StartCharge::new(3_520, 160);
/// What `LOG` charges as history.
const HISTORY_COST: u64 = 100;
/// A value call's history allowance on a chain that gives one.
const ALLOWANCE: u64 = 160;
/// The pool nobody set.
const UNLIMITED: u64 = u64::MAX;
/// The pool `LIMIT` sets.
const LIMITED: u64 = 1_000;

/// The account the transaction calls.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Draws and stops.
const OK: Address = address!("0x0000000000000000000000000000000000000c00");
/// Draws and reverts.
const REVERTER: Address = address!("0x000000000000000000000000000000000000c0fd");
/// Draws and halts.
const HALTER: Address = address!("0x000000000000000000000000000000000000c0fe");
/// Draws, calls [`OK`] and reverts.
const NESTER: Address = address!("0x000000000000000000000000000000000000c0e5");
/// Draws more than any pool holds.
const DRAINER: Address = address!("0x000000000000000000000000000000000000d7a1");
/// Charges history after drawing its allowance, and stops.
const LOGGER: Address = address!("0x000000000000000000000000000000000000c109");

/// The test's instructions, on opcodes the EVM does not assign.
const DRAW: u8 = 0x0c;
const DECLARE: u8 = 0x0d;
const LOG: u8 = 0x0e;
const DRAIN: u8 = 0x0f;
const LIMIT: u8 = 0x10;

const fn draw(context: InstructionContext<'_, Ctx, EthInterpreter>) -> InstructionExecResult {
    if context.interpreter.gas.record_bytes(DRAW_BYTES) {
        Ok(())
    } else {
        Err(InstructionResult::OutOfGas)
    }
}

const fn drain(context: InstructionContext<'_, Ctx, EthInterpreter>) -> InstructionExecResult {
    if context.interpreter.gas.record_bytes(u64::MAX) {
        Ok(())
    } else {
        Err(InstructionResult::OutOfGas)
    }
}

/// Sets the frame's pool to [`LIMITED`], as a consumer sets the transaction's.
const fn limit(context: InstructionContext<'_, Ctx, EthInterpreter>) -> InstructionExecResult {
    context.interpreter.gas.set_bytes_remaining(LIMITED);
    Ok(())
}

/// Has the start hook charge and declare [`DECLARED`] at the start that follows.
const fn declare(context: InstructionContext<'_, Ctx, EthInterpreter>) -> InstructionExecResult {
    context.host.declare_next = true;
    Ok(())
}

/// What a consumer's write hook does for a log.
const fn log(context: InstructionContext<'_, Ctx, EthInterpreter>) -> InstructionExecResult {
    let gas = &mut context.interpreter.gas;
    let rest = gas.draw_history_allowance(HISTORY_COST);
    if gas.record_history_cost(rest) {
        Ok(())
    } else {
        Err(InstructionResult::OutOfGas)
    }
}

/// The mainnet [`Context`] with a start hook that does what a consumer's does for a start
/// `DECLARE` marked: charges [`DECLARED`] on the caller's gas and declares it on the start.
struct DeclaringContext {
    inner: MainnetContext<Db>,
    /// Set by `DECLARE`, taken by the next start.
    declare_next: bool,
}

impl ContextTr for DeclaringContext {
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

impl ContextSetters for DeclaringContext {
    fn set_tx(&mut self, tx: Self::Tx) {
        self.inner.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.inner.set_block(block);
    }
}

impl Host for DeclaringContext {
    fn frame_starting(&mut self, start: &mut FrameStart, gas: &mut GasTracker) -> bool {
        if core::mem::take(&mut self.declare_next) {
            assert!(gas.record_history_cost(DECLARED.history));
            assert!(gas.record_bytes(DECLARED.bytes));
            start.charge = DECLARED;
        }
        true
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

/// A frame's figures as it ended, before its caller settled it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Ended {
    result: InstructionResult,
    pool: u64,
    spent: i64,
    crossing: bool,
    history: i64,
    allowance: u64,
    charged_start: StartCharge,
}

impl Ended {
    const fn stopped(pool: u64, spent: i64) -> Self {
        Self::with(InstructionResult::Stop, pool, spent)
    }

    const fn with(result: InstructionResult, pool: u64, spent: i64) -> Self {
        Self {
            result,
            pool,
            spent,
            crossing: false,
            history: 0,
            allowance: 0,
            charged_start: StartCharge::new(0, 0),
        }
    }
}

/// What the inspector does with a call to [`OK`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Answer {
    /// Lets the frame run.
    #[default]
    Runs,
    /// Answers the start with a revert built by `CallOutcome::new`, as a tool builds one.
    AsBuilt,
    /// Answers the start with a revert that carries the input's share and start charge.
    WithShareAndCharge,
    /// Lets the frame run, and rewrites its success into a revert at `call_end`.
    RewrittenIntoRevert,
}

/// Every frame's input as it started (share, start charge, allowance) and its outcome as it
/// ended; and what the inspector does with a call to [`OK`].
#[derive(Default)]
struct Figures {
    starts: Vec<(u64, StartCharge, u64)>,
    ends: Vec<Ended>,
    answer: Answer,
}

impl Figures {
    fn end(&mut self, result: InstructionResult, gas: &Gas, charged: StartCharge) {
        self.ends.push(Ended {
            result,
            pool: gas.bytes_remaining(),
            spent: gas.bytes_spent(),
            crossing: gas.bytes_crossing(),
            history: gas.history_gas_spent(),
            allowance: gas.history_allowance(),
            charged_start: charged,
        });
    }
}

impl Inspector<Ctx, EthInterpreter> for Figures {
    fn call(&mut self, _context: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        self.starts.push((
            inputs.bytes_share,
            inputs.charged_start,
            inputs.history_allowance,
        ));
        let answers = matches!(self.answer, Answer::AsBuilt | Answer::WithShareAndCharge);
        if !answers || inputs.target_address != OK {
            return None;
        }
        let revert = InterpreterResult::new(
            InstructionResult::Revert,
            Bytes::new(),
            Gas::new(inputs.gas_limit),
        );
        let mut outcome = CallOutcome::new(revert, inputs.return_memory_offset.clone());
        if self.answer == Answer::WithShareAndCharge {
            outcome.result.gas.set_bytes_remaining(inputs.bytes_share);
            outcome.charged_start = inputs.charged_start;
        }
        Some(outcome)
    }

    fn create(&mut self, _context: &mut Ctx, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        self.starts
            .push((inputs.bytes_share(), inputs.charged_start(), 0));
        None
    }

    fn call_end(&mut self, _context: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
        if self.answer == Answer::RewrittenIntoRevert && inputs.target_address == OK {
            outcome.result.result = InstructionResult::Revert;
        }
        self.end(
            *outcome.instruction_result(),
            &outcome.result.gas,
            outcome.charged_start,
        );
    }

    fn create_end(
        &mut self,
        _context: &mut Ctx,
        _inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        self.end(
            *outcome.instruction_result(),
            &outcome.result.gas,
            outcome.charged_start,
        );
    }
}

/// `CALL(100_000, to, value, 0, 0, 0, 0)`, dropping the success flag.
fn call(to: Address, value: u8) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH1, value, PUSH20];
    code.extend_from_slice(to.as_slice());
    code.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, CALL, POP]);
    code
}

/// `CREATE` of `initcode`, at most 32 bytes, endowed with nothing, dropping the address.
fn create(initcode: &[u8]) -> Vec<u8> {
    let len = initcode.len() as u8;
    let mut code = vec![PUSH0 + len];
    code.extend_from_slice(initcode);
    code.extend_from_slice(&[
        PUSH0,
        MSTORE,
        PUSH1,
        len,
        PUSH1,
        32 - len,
        PUSH0,
        CREATE,
        POP,
    ]);
    code
}

fn ends(mut code: Vec<u8>) -> Vec<u8> {
    code.push(STOP);
    code
}

/// A transaction to [`PARENT`], holding `parent`, on the accounts every test knows.
struct Run {
    parent: Vec<u8>,
    allowance: u64,
}

impl Run {
    const fn new(parent: Vec<u8>) -> Self {
        Self {
            parent,
            allowance: 0,
        }
    }

    const fn allowance(mut self, allowance: u64) -> Self {
        self.allowance = allowance;
        self
    }

    fn evm(&self) -> PoolEvm {
        let mut db = Db::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo::from_balance(U256::from(10u128.pow(21))),
        );
        let mut nester = vec![DRAW];
        nester.extend(call(OK, 0));
        nester.extend_from_slice(&[PUSH0, PUSH0, REVERT]);
        let accounts: [(Address, Vec<u8>); 7] = [
            (PARENT, self.parent.clone()),
            (OK, vec![DRAW, STOP]),
            (REVERTER, vec![DRAW, PUSH0, PUSH0, REVERT]),
            (HALTER, vec![DRAW, INVALID]),
            (NESTER, nester),
            (DRAINER, vec![DRAIN, STOP]),
            (LOGGER, vec![LOG, STOP]),
        ];
        for (address, code) in accounts {
            let info = AccountInfo::from_balance(U256::from(10))
                .with_nonce(1)
                .with_code(Bytecode::new_raw(Bytes::from(code)));
            db.insert_account_info(address, info);
        }
        let allowance = self.allowance;
        let inner = Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SPEC);
            cfg.gas_params
                .override_gas([(GasId::call_history_allowance(), allowance)]);
        });
        let ctx = DeclaringContext {
            inner,
            declare_next: false,
        };
        let mut instructions = EthInstructions::new_mainnet_with_spec(SPEC);
        instructions.insert_instruction(DRAW, Instruction::new(draw), 0);
        instructions.insert_instruction(DRAIN, Instruction::new(drain), 0);
        instructions.insert_instruction(LIMIT, Instruction::new(limit), 0);
        instructions.insert_instruction(DECLARE, Instruction::new(declare), 0);
        instructions.insert_instruction(LOG, Instruction::new(log), 0);
        Evm::new_with_inspector(
            ctx,
            Figures::default(),
            instructions,
            EthPrecompiles::new(SPEC),
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

    /// The transaction with `transact` and with `inspect_tx`, which must end the same way.
    /// Returns the outcome and the figures the inspector saw.
    fn on_both_paths(&self) -> (ResultAndState, Figures) {
        let mut plain = self.evm();
        let transacted = plain.transact(Self::tx()).expect("the transaction runs");
        let mut inspected = self.evm();
        let inspected_outcome = inspected
            .inspect_tx(Self::tx())
            .expect("the transaction runs");
        assert_eq!(inspected_outcome, transacted, "both paths end the same way");
        (transacted, inspected.inspector)
    }

    /// The transaction with `inspect_tx` alone, the inspector doing `answer` with a call to
    /// [`OK`]. Returns the outcome and the figures the inspector saw.
    fn inspected(&self, answer: Answer) -> (ResultAndState, Figures) {
        let mut evm = self.evm();
        evm.inspector.answer = answer;
        let outcome = evm.inspect_tx(Self::tx()).expect("the transaction runs");
        (outcome, evm.inspector)
    }
}

/// Both paths forward the whole pool, less the draws so far, and settle a child's return the
/// same way: what it left on success, the whole share on revert and on halt, a nested success's
/// draws undone with its caller's revert.
#[test]
fn test_both_paths_forward_and_settle_the_pool_at_depth() {
    let mut parent = vec![DRAW];
    for child in [OK, REVERTER, HALTER, NESTER] {
        parent.extend(call(child, 0));
    }
    let (outcome, figures) = Run::new(ends(parent)).on_both_paths();

    assert!(outcome.result.is_success(), "{:?}", outcome.result);
    let less = |draws: u64| UNLIMITED - draws * DRAW_BYTES;
    let share = |pool| (pool, StartCharge::default(), 0);
    assert_eq!(
        figures.starts,
        [
            share(less(0)),
            share(less(1)),
            share(less(2)),
            share(less(2)),
            share(less(2)),
            share(less(3)),
        ]
    );
    assert_eq!(
        figures.ends,
        [
            Ended::stopped(less(2), 100),
            Ended::with(InstructionResult::Revert, less(3), 100),
            Ended::with(InstructionResult::InvalidFEOpcode, less(3), 100),
            Ended::stopped(less(4), 100),
            Ended::with(InstructionResult::Revert, less(4), 200),
            Ended::stopped(less(2), 200),
        ]
    );
}

/// A draw the pool cannot pay records a crossing the inspector reads off the frame's outcome,
/// with the pool as the draw found it, on both paths.
#[test]
fn test_both_paths_record_a_failed_draw() {
    let mut parent = vec![DRAW];
    parent.extend(call(DRAINER, 0));
    let (outcome, figures) = Run::new(ends(parent)).on_both_paths();

    assert!(outcome.result.is_success(), "{:?}", outcome.result);
    assert_eq!(
        figures.ends,
        [
            Ended {
                crossing: true,
                ..Ended::with(InstructionResult::OutOfGas, UNLIMITED - DRAW_BYTES, 0)
            },
            Ended::stopped(UNLIMITED - DRAW_BYTES, DRAW_BYTES as i64),
        ]
    );
}

/// A declared start charge rides on the input and the outcome, and the caller gets it back whole
/// when the frame reverts, halts or is a creation that reverts, and keeps it when the frame
/// succeeds, on both paths.
#[test]
fn test_both_paths_refund_the_start_charge_when_the_frame_fails() {
    let reverting_initcode = [PUSH0, PUSH0, REVERT];
    for (name, start, fails) in [
        ("a child that stops", call(OK, 0), false),
        ("a child that reverts", call(REVERTER, 0), true),
        ("a child that halts", call(HALTER, 0), true),
        ("a creation that reverts", create(&reverting_initcode), true),
    ] {
        let mut parent = vec![DECLARE];
        parent.extend(start);
        let (outcome, figures) = Run::new(ends(parent)).on_both_paths();

        assert!(outcome.result.is_success(), "{name}: {:?}", outcome.result);
        assert_eq!(
            figures.starts[1],
            (UNLIMITED - DECLARED.bytes, DECLARED, 0),
            "{name}: the charge rides on the input"
        );
        assert_eq!(
            figures.ends[0].charged_start, DECLARED,
            "{name}: and on the outcome"
        );
        let parent_end = figures.ends.last().unwrap();
        if fails {
            assert_eq!(
                (parent_end.pool, parent_end.spent, parent_end.history),
                (UNLIMITED, 0, 0),
                "{name}: refunded whole"
            );
        } else {
            let drawn = DECLARED.bytes + DRAW_BYTES;
            assert_eq!(
                (parent_end.pool, parent_end.spent, parent_end.history),
                (UNLIMITED - drawn, drawn as i64, DECLARED.history as i64),
                "{name}: kept"
            );
        }
    }
}

/// A value call's child starts with the schedule's allowance on both paths, and a consumer's
/// charge takes it before the budget.
#[test]
fn test_both_paths_give_a_value_calls_child_the_allowance() {
    let mut parent = call(LOGGER, 1);
    parent.extend(call(LOGGER, 0));
    let (outcome, figures) = Run::new(ends(parent)).allowance(ALLOWANCE).on_both_paths();

    assert!(outcome.result.is_success(), "{:?}", outcome.result);
    let allowances: Vec<u64> = figures.starts.iter().map(|start| start.2).collect();
    assert_eq!(allowances, [0, ALLOWANCE, 0]);
    assert_eq!(
        figures.ends,
        [
            Ended {
                allowance: ALLOWANCE - HISTORY_COST,
                ..Ended::stopped(UNLIMITED, 0)
            },
            Ended {
                history: HISTORY_COST as i64,
                ..Ended::stopped(UNLIMITED, 0)
            },
            Ended {
                history: HISTORY_COST as i64,
                ..Ended::stopped(UNLIMITED, 0)
            },
        ]
    );
}

/// [`PARENT`] limits its pool to [`LIMITED`] and calls [`OK`] with a start charge declared, on
/// the inspected path alone, the inspector doing `answer`. Returns the share the call was given,
/// [`OK`]'s outcome as the inspector left it, and [`PARENT`]'s pool, net and history after the
/// settlement.
fn limited_call_to_ok(answer: Answer) -> (u64, Ended, (u64, i64, i64)) {
    let parent = ends([vec![LIMIT, DECLARE], call(OK, 0)].concat());
    let (outcome, figures) = Run::new(parent).inspected(answer);

    assert!(
        outcome.result.is_success(),
        "{answer:?}: {:?}",
        outcome.result
    );
    let [ok, parent_end] = &figures.ends[..] else {
        panic!("{answer:?}: {} ends", figures.ends.len());
    };
    assert_eq!(figures.starts[1].1, DECLARED, "{answer:?}");
    (
        figures.starts[1].0,
        ok.clone(),
        (parent_end.pool, parent_end.spent, parent_end.history),
    )
}

/// An outcome the inspector builds to answer a start settles as built, as revm settles its own
/// upfront state charge on such an outcome. `CallOutcome::new` carries no start charge and an
/// unlimited pool, so a caller whose start the inspector answers with a revert keeps the start
/// charge, and its limited pool, plus the unlimited one absorbed, saturates to `u64::MAX`. An
/// answer that carries the input's share and charge settles as the frame's own revert would: the
/// caller gets the share and the charge back.
#[test]
fn test_an_outcome_the_inspector_builds_settles_as_built() {
    let share = LIMITED - DECLARED.bytes;
    for (answer, answered, kept) in [
        (
            Answer::AsBuilt,
            Ended::with(InstructionResult::Revert, UNLIMITED, 0),
            (UNLIMITED, DECLARED.bytes as i64, DECLARED.history as i64),
        ),
        (
            Answer::WithShareAndCharge,
            Ended {
                charged_start: DECLARED,
                ..Ended::with(InstructionResult::Revert, share, 0)
            },
            (LIMITED, 0, 0),
        ),
    ] {
        assert_eq!(
            limited_call_to_ok(answer),
            (share, answered, kept),
            "{answer:?}: the share, the answer as built, the caller after the settlement"
        );
    }
}

/// A frame's outcome carries its input's start charge, so a success the inspector rewrites into a
/// revert at `call_end` refunds the charge and rolls the frame's draws back, as the frame's own
/// revert would; the frame left alone keeps both.
#[test]
fn test_a_success_the_inspector_rewrites_into_a_revert_refunds_the_start_charge() {
    let share = LIMITED - DECLARED.bytes;
    let drawn = DECLARED.bytes + DRAW_BYTES;
    for (answer, result, kept) in [
        (
            Answer::Runs,
            InstructionResult::Stop,
            (LIMITED - drawn, drawn as i64, DECLARED.history as i64),
        ),
        (
            Answer::RewrittenIntoRevert,
            InstructionResult::Revert,
            (LIMITED, 0, 0),
        ),
    ] {
        let ended = Ended {
            charged_start: DECLARED,
            ..Ended::with(result, share - DRAW_BYTES, DRAW_BYTES as i64)
        };
        assert_eq!(
            limited_call_to_ok(answer),
            (share, ended, kept),
            "{answer:?}: the share, the frame's outcome as the inspector left it, the caller after \
             the settlement"
        );
    }
}
