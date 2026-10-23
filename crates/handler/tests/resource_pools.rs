//! The resource pools, the start charge and the history allowance, as the mainnet handler forwards
//! and settles them.
//!
//! A consumer sets the transaction's pools on the gas `tx_gas` returns; every frame then starts
//! with the share its caller's instruction forwarded, and `handle_reservoir_remaining_gas` settles
//! what it left: its remaining pool when it succeeded, the whole share when it reverted or halted.
//! A start charge a hook declared rides on the frame's input and is refunded whole when the frame
//! fails. The handler here is the mainnet one, observed through overrides that leave its behaviour
//! alone, on the mainnet context with a start hook of the test's own ([`DeclaringContext`]), and
//! the frames draw the pool through instructions of the test's own, which stand in for a
//! consumer's hooks: `DRAW` draws bytes, `DECLARE` has the start hook charge and declare a start
//! charge at the next start, and `LOG` charges history after drawing the frame's allowance, as a
//! consumer's write hook would. The context's code-deposit admission hook draws deployed code's
//! bytes, and the precompile provider draws on the answers it builds, when a test asks them to.

use bytecode::opcode::{
    CALL, CREATE, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3, RETURN, REVERT, STOP,
};
use context::{
    result::{EVMError, ExecutionResult},
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Journal, LocalContext,
    TxEnv,
};
use context_interface::{
    cfg::{gas::InitialAndFloorGas, GasId, GasParams, StateGasSite},
    context::CodeDeposit,
    host::{FrameStart, LoadError},
    journaled_state::AccountInfoLoad,
    Database, Host,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, CallInputs, FrameInput, GasTracker,
    Instruction, InstructionContext, InstructionExecResult, InstructionResult, InterpreterResult,
    PoolAmounts, PoolId, SStoreResult, SelfDestructResult, StartCharge, StateLoad,
};
use primitives::{
    address, constants::CALL_STACK_LIMIT, hardfork::SpecId, Address, AddressSet, Bytes, Log,
    StorageKey, StorageValue, TxKind, B256, U256,
};
use revm_handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr, FrameResult, Handler,
    ItemOrResult, MainContext, MainnetContext, MainnetHandler, PrecompileProvider,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = DeclaringContext;
type TestEvm = Evm<
    TestContext,
    (),
    EthInstructions<EthInterpreter, TestContext>,
    DrawingPrecompiles,
    EthFrame,
>;
type TestError = EVMError<DbError>;
type Mainnet = MainnetHandler<TestEvm, TestError, EthFrame<EthInterpreter>>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 3_000_000;
/// The pool the transaction's frame starts with.
const POOL: u64 = 1_000;
/// What `DRAW` draws.
const DRAW_BYTES: u64 = 100;
/// The pool the tests draw: any of the tracker's pools behaves the same.
const FIRST: PoolId = PoolId::new(0);
/// What the start hook charges and declares after `DECLARE`.
const DECLARED: StartCharge = StartCharge::new(3_520, PoolAmounts::ZERO.with(FIRST, 160));
/// What `LOG` charges as history.
const HISTORY_COST: u64 = 100;
/// A value call's history allowance on a chain that gives one.
const ALLOWANCE: u64 = 160;

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
/// Charges history after drawing its allowance, and stops.
const LOGGER: Address = address!("0x000000000000000000000000000000000000c109");
/// An account that does not exist.
const EMPTY: Address = address!("0x00000000000000000000000000000000000e3970");
/// The `ECRECOVER` precompile, which costs 3,000.
const ECRECOVER: Address = address!("0x0000000000000000000000000000000000000001");

/// The test's instructions, on opcodes the EVM does not assign.
const DRAW: u8 = 0x0c;
const DECLARE: u8 = 0x0d;
const LOG: u8 = 0x0e;

/// Draws [`DRAW_BYTES`] from the frame's pool; a draw the pool cannot pay is the frame's stop.
const fn draw(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    let mut index = 0;
    while index < PoolId::COUNT {
        if !context
            .interpreter
            .gas
            .record_pool(PoolId::ALL[index], DRAW_BYTES)
        {
            return Err(InstructionResult::OutOfGas);
        }
        index += 1;
    }
    Ok(())
}

/// Has the start hook charge and declare [`DECLARED`] at the start that follows.
const fn declare(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    context.host.declare_next = true;
    Ok(())
}

/// What a consumer's write hook does for a log: draws the frame's allowance for [`HISTORY_COST`]
/// and charges the rest as history.
const fn log(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    let gas = &mut context.interpreter.gas;
    let rest = gas.draw_history_allowance(HISTORY_COST);
    if gas.record_history_cost(rest) {
        Ok(())
    } else {
        Err(InstructionResult::OutOfGas)
    }
}

/// A frame's pool figures as it returned, before its caller settled it: its result, its pool,
/// its net draw, whether a draw failed, its history gas and its allowance.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Returned {
    result: InstructionResult,
    pool: u64,
    spent: i64,
    crossing: bool,
    history: i64,
    allowance: u64,
}

impl Returned {
    const fn of(result: &FrameResult) -> Self {
        let gas = result.gas();
        Self {
            result: result.instruction_result(),
            pool: gas.pool_remaining(FIRST),
            spent: gas.pool_spent(FIRST),
            crossing: gas.pool_crossing(FIRST),
            history: gas.history_gas_spent(),
            allowance: gas.history_allowance(),
        }
    }

    const fn stopped(pool: u64, spent: i64) -> Self {
        Self::ended(InstructionResult::Stop, pool, spent)
    }

    const fn ended(result: InstructionResult, pool: u64, spent: i64) -> Self {
        Self {
            result,
            pool,
            spent,
            crossing: false,
            history: 0,
            allowance: 0,
        }
    }
}

/// What the code-deposit admission hook does with a deposit.
#[derive(Clone, Copy, Debug)]
enum Deposit {
    /// Admits it and touches nothing, as the default hook does.
    Untouched,
    /// Draws the code's bytes from the creating frame's pool and admits it.
    Drawn,
    /// Draws the code's bytes from the creating frame's pool and refuses it.
    DrawnAndRefused,
}

/// The mainnet [`Context`] with a start hook that does what a consumer's does for a start
/// `DECLARE` marked: charges [`DECLARED`] on the caller's gas and declares it on the start; and an
/// admission hook that does with a code deposit what [`Deposit`] says.
struct DeclaringContext {
    inner: MainnetContext<Db>,
    /// Set by `DECLARE`, taken by the next start.
    declare_next: bool,
    deposit: Deposit,
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

    fn admit_code_deposit(
        &mut self,
        deposit: &CodeDeposit<'_>,
        gas: &mut GasTracker,
    ) -> Result<(), Bytes> {
        let refuses = match self.deposit {
            Deposit::Untouched => return Ok(()),
            Deposit::Drawn => false,
            Deposit::DrawnAndRefused => true,
        };
        assert!(gas.record_pool(FIRST, deposit.code.len() as u64));
        if refuses {
            Err(Bytes::new())
        } else {
            Ok(())
        }
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
            assert!(gas.record_pool(FIRST, DECLARED.pools.get(FIRST)));
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

/// [`EthPrecompiles`], whose answers the provider draws on when told to: it starts an answer's
/// pool at the share the input carries and draws `draw` bytes from it, as a provider that appends
/// bytes for a call would. Told not to `seed`, it draws `draw` from every pool of the answer as
/// [`EthPrecompiles`] built it, unlimited, without starting them at the shares. With nothing to
/// draw it is [`EthPrecompiles`].
struct DrawingPrecompiles {
    inner: EthPrecompiles,
    draw: u64,
    seed: bool,
}

impl PrecompileProvider<TestContext> for DrawingPrecompiles {
    type Output = InterpreterResult;

    fn set_spec(&mut self, spec: SpecId) -> bool {
        PrecompileProvider::<TestContext>::set_spec(&mut self.inner, spec)
    }

    fn run(
        &mut self,
        context: &mut TestContext,
        inputs: &CallInputs,
    ) -> Result<Option<InterpreterResult>, String> {
        let mut answer = self.inner.run(context, inputs)?;
        if let Some(answer) = answer.as_mut().filter(|_| self.draw > 0) {
            if self.seed {
                answer
                    .gas
                    .set_pool_remaining(FIRST, inputs.pool_shares.get(FIRST));
                assert!(answer.gas.record_pool(FIRST, self.draw));
            } else {
                for pool in PoolId::ALL {
                    assert!(answer.gas.record_pool(pool, self.draw));
                }
            }
        }
        Ok(answer)
    }

    fn warm_addresses(&self) -> &AddressSet {
        self.inner.warm_addresses()
    }
}

/// The figures of a frame's input as it started: its share, its start charge and its allowance.
type Started = (u64, StartCharge, u64);

/// Every pool's share on a frame's input.
fn pool_shares(input: &FrameInput) -> PoolAmounts {
    match input {
        FrameInput::Call(inputs) => inputs.pool_shares,
        FrameInput::Create(inputs) => inputs.pool_shares(),
        FrameInput::Empty => unreachable!(),
    }
}

fn started(input: &FrameInput) -> Started {
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

/// The transaction's gas after the first frame settled into it: its pool, its net draw and its
/// history gas.
type Settled = (u64, i64, i64);

/// The mainnet handler with the consumer's part: a pool set on the transaction's gas and a start
/// charge for the transaction's frame, and a record of every frame's start and return.
struct PoolHandler {
    mainnet: Mainnet,
    /// The pool the transaction's gas gets, if any.
    pool: Option<u64>,
    /// Whether the transaction's gas is charged [`DECLARED`] for its frame, declared on the
    /// frame's input.
    declare: bool,
    /// Whether every frame below the transaction's starts one past the call-stack limit, where a
    /// 1,025th nested start would.
    too_deep: bool,
    starts: Vec<Started>,
    returned: Vec<Returned>,
    settled: Option<Settled>,
    /// Every pool's share as each frame started.
    pool_starts: Vec<PoolAmounts>,
    /// Every pool of the transaction's gas, `(remaining, spent)`, after the first frame settled
    /// into it.
    settled_pools: Option<[(u64, i64); PoolId::COUNT]>,
}

impl PoolHandler {
    fn new(pool: Option<u64>) -> Self {
        Self {
            mainnet: Mainnet::default(),
            pool,
            declare: false,
            too_deep: false,
            starts: Vec::new(),
            returned: Vec::new(),
            settled: None,
            pool_starts: Vec::new(),
            settled_pools: None,
        }
    }
}

impl Handler for PoolHandler {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = context::result::HaltReason;

    /// The mainnet gas, with the consumer's pool and, if told to, the start charge of the
    /// transaction's frame, charged here as a consumer charges the body's records.
    fn tx_gas(&self, evm: &mut TestEvm, init_and_floor_gas: &InitialAndFloorGas) -> GasTracker {
        let mut gas = self.mainnet.tx_gas(evm, init_and_floor_gas);
        if let Some(pool) = self.pool {
            for each in PoolId::ALL {
                gas.set_pool_remaining(each, pool);
            }
        }
        if self.declare {
            assert!(gas.record_history_cost(DECLARED.history));
            assert!(gas.record_pool(FIRST, DECLARED.pools.get(FIRST)));
        }
        gas
    }

    /// The mainnet input, with the start charge `tx_gas` charged declared on it, if told to.
    fn first_frame_input(
        &mut self,
        evm: &mut TestEvm,
        gas: &mut GasTracker,
    ) -> Result<Option<FrameInit>, TestError> {
        let mut init = self.mainnet.first_frame_input(evm, gas)?;
        if let (true, Some(init)) = (self.declare, &mut init) {
            match &mut init.frame_input {
                FrameInput::Call(inputs) => inputs.charged_start = DECLARED,
                FrameInput::Create(inputs) => inputs.set_charged_start(DECLARED),
                FrameInput::Empty => unreachable!(),
            }
        }
        Ok(init)
    }

    /// The default loop, recording each frame's input as it starts and its result as it
    /// returns, before its caller settles it, and starting the frames below the transaction's
    /// past the call-stack limit if told to.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        self.starts.push(started(&first_frame_input.frame_input));
        self.pool_starts
            .push(pool_shares(&first_frame_input.frame_input));
        if let ItemOrResult::Result(result) = evm.frame_init(first_frame_input)? {
            self.returned.push(Returned::of(&result));
            return Ok(result);
        }
        loop {
            let result = match evm.frame_run()? {
                ItemOrResult::Item(mut init) => {
                    self.starts.push(started(&init.frame_input));
                    self.pool_starts.push(pool_shares(&init.frame_input));
                    if self.too_deep {
                        init.depth = CALL_STACK_LIMIT as usize + 1;
                    }
                    match evm.frame_init(init)? {
                        ItemOrResult::Item(_) => continue,
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(mut result) => {
                    evm.frame_run_result(&mut result)?;
                    result
                }
            };
            self.returned.push(Returned::of(&result));
            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }

    /// The default, recording the transaction's gas as the first frame left it.
    fn post_execution(
        &self,
        evm: &mut TestEvm,
        exec_result: &mut FrameResult,
        init_and_floor_gas: InitialAndFloorGas,
        eip7702_refund: i64,
    ) -> Result<context::result::ResultGas, TestError> {
        let gas = exec_result.gas();
        // `post_execution` takes `&self`; the record goes through a cell.
        SETTLED.with(|settled| {
            settled.set(Some((
                gas.pool_remaining(FIRST),
                gas.pool_spent(FIRST),
                gas.history_gas_spent(),
            )))
        });
        SETTLED_POOLS.with(|settled| {
            settled.set(Some(
                PoolId::ALL.map(|pool| (gas.pool_remaining(pool), gas.pool_spent(pool))),
            ))
        });
        self.mainnet
            .post_execution(evm, exec_result, init_and_floor_gas, eip7702_refund)
    }
}

thread_local! {
    static SETTLED: core::cell::Cell<Option<Settled>> = const { core::cell::Cell::new(None) };
    static SETTLED_POOLS: core::cell::Cell<Option<[(u64, i64); PoolId::COUNT]>> =
        const { core::cell::Cell::new(None) };
}

/// `CALL(100_000, to, value, 0, 0, 0, 0)`, dropping the success flag.
fn call(to: Address, value: u8) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH1, value, PUSH20];
    code.extend_from_slice(to.as_slice());
    code.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, CALL, POP]);
    code
}

/// `CALL(10, to, 0, 0, 0, 0, 0)`: too little gas for any precompile, dropping the success flag.
fn starved_call(to: Address) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20];
    code.extend_from_slice(to.as_slice());
    code.extend_from_slice(&[PUSH1, 10, CALL, POP]);
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

/// A transaction on the accounts every test knows: a call to [`PARENT`], which holds `parent`, or
/// a creation with `parent` as its init code ([`Run::creation`]).
struct Run {
    parent: Vec<u8>,
    parent_balance: u64,
    parent_nonce: u64,
    /// `call_history_allowance` in the schedule.
    allowance: u64,
    deposit: Deposit,
    /// What the precompile provider draws on each answer.
    precompile_draw: u64,
    /// Whether the provider starts an answer's pool at the share before it draws.
    precompile_seeds: bool,
    /// The per cent of [`FIRST`] a child receives, if not the default: the schedule's
    /// `pool_kept_percent` is the rest.
    forward_percent: Option<u64>,
    /// The per cent of each pool a child receives, if not the default.
    pool_percents: Option<[u64; PoolId::COUNT]>,
    /// Whether the transaction creates a contract from `parent` instead of calling [`PARENT`].
    creates: bool,
}

impl Run {
    const fn new(parent: Vec<u8>) -> Self {
        Self {
            parent,
            parent_balance: 10,
            parent_nonce: 1,
            allowance: 0,
            deposit: Deposit::Untouched,
            precompile_draw: 0,
            precompile_seeds: true,
            forward_percent: None,
            pool_percents: None,
            creates: false,
        }
    }

    /// A transaction that creates a contract from `initcode`.
    const fn creation(initcode: Vec<u8>) -> Self {
        let mut run = Self::new(initcode);
        run.creates = true;
        run
    }

    const fn pool_percents(mut self, percents: [u64; PoolId::COUNT]) -> Self {
        self.pool_percents = Some(percents);
        self
    }

    const fn forward_percent(mut self, percent: u64) -> Self {
        self.forward_percent = Some(percent);
        self
    }

    const fn precompile_draw(mut self, draw: u64) -> Self {
        self.precompile_draw = draw;
        self
    }

    /// The provider draws `draw` from every pool of each answer as built, without starting the
    /// pools at the shares first.
    const fn unseeded_precompile_draw(mut self, draw: u64) -> Self {
        self.precompile_draw = draw;
        self.precompile_seeds = false;
        self
    }

    const fn deposit(mut self, deposit: Deposit) -> Self {
        self.deposit = deposit;
        self
    }

    const fn penniless(mut self) -> Self {
        self.parent_balance = 0;
        self
    }

    const fn nonce(mut self, nonce: u64) -> Self {
        self.parent_nonce = nonce;
        self
    }

    const fn allowance(mut self, allowance: u64) -> Self {
        self.allowance = allowance;
        self
    }

    fn db(&self) -> Db {
        let mut db = Db::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo::from_balance(U256::from(10u128.pow(21))),
        );
        let mut nester = vec![DRAW];
        nester.extend(call(OK, 0));
        nester.extend_from_slice(&[PUSH0, PUSH0, REVERT]);
        let accounts: [(Address, Vec<u8>, u64, u64); 6] = [
            (
                PARENT,
                self.parent.clone(),
                self.parent_balance,
                self.parent_nonce,
            ),
            (OK, vec![DRAW, STOP], 0, 1),
            (REVERTER, vec![DRAW, PUSH0, PUSH0, REVERT], 0, 1),
            (HALTER, vec![DRAW, INVALID], 0, 1),
            (NESTER, nester, 0, 1),
            (LOGGER, vec![LOG, STOP], 0, 1),
        ];
        for (address, code, balance, nonce) in accounts {
            let info = AccountInfo::from_balance(U256::from(balance))
                .with_nonce(nonce)
                .with_code(Bytecode::new_raw(Bytes::from(code)));
            db.insert_account_info(address, info);
        }
        db
    }

    fn evm(&self) -> TestEvm {
        let allowance = self.allowance;
        let forward_percent = self.forward_percent;
        let pool_percents = self.pool_percents;
        let inner = Context::mainnet()
            .with_db(self.db())
            .modify_cfg_chained(|cfg| {
                cfg.set_spec_and_mainnet_gas_params(SPEC);
                cfg.gas_params
                    .override_gas([(GasId::call_history_allowance(), allowance)]);
                if let Some(percents) = pool_percents {
                    for (pool, percent) in PoolId::ALL.into_iter().zip(percents) {
                        cfg.gas_params
                            .override_gas([(GasId::pool_kept_percent(pool), 100 - percent)]);
                    }
                }
                if let Some(percent) = forward_percent {
                    cfg.gas_params
                        .override_gas([(GasId::pool_kept_percent(FIRST), 100 - percent)]);
                }
            });
        let ctx = DeclaringContext {
            inner,
            declare_next: false,
            deposit: self.deposit,
        };
        let mut instructions = EthInstructions::new_mainnet_with_spec(SPEC);
        instructions.insert_instruction(DRAW, Instruction::new(draw), 0);
        instructions.insert_instruction(DECLARE, Instruction::new(declare), 0);
        instructions.insert_instruction(LOG, Instruction::new(log), 0);
        let precompiles = DrawingPrecompiles {
            inner: EthPrecompiles::new(SPEC),
            draw: self.precompile_draw,
            seed: self.precompile_seeds,
        };
        let mut evm: TestEvm = Evm::new(ctx, instructions, precompiles);
        let (kind, data) = if self.creates {
            (TxKind::Create, Bytes::from(self.parent.clone()))
        } else {
            (TxKind::Call(PARENT), Bytes::new())
        };
        evm.ctx.set_tx(
            TxEnv::builder_for_bench()
                .kind(kind)
                .data(data)
                .gas_price(0)
                .gas_limit(GAS_LIMIT)
                .build_fill(),
        );
        evm
    }

    /// Runs the transaction with `handler` and returns its result and the handler, with what it
    /// recorded.
    fn with(&self, mut handler: PoolHandler) -> (ExecutionResult, PoolHandler) {
        SETTLED.with(|settled| settled.set(None));
        SETTLED_POOLS.with(|settled| settled.set(None));
        let result = handler.run(&mut self.evm()).expect("the transaction runs");
        handler.settled = SETTLED.with(|settled| settled.get());
        handler.settled_pools = SETTLED_POOLS.with(|settled| settled.get());
        (result, handler)
    }

    /// Runs the transaction with the mainnet handler as it is.
    fn on_mainnet(&self) -> ExecutionResult {
        Mainnet::default()
            .run(&mut self.evm())
            .expect("the transaction runs")
    }
}

/// [`PARENT`] draws, then calls a child that stops, one that reverts, one that halts and one
/// that calls a child of its own and reverts.
fn at_depth() -> Run {
    let mut parent = vec![DRAW];
    for child in [OK, REVERTER, HALTER, NESTER] {
        parent.extend(call(child, 0));
    }
    Run::new(ends(parent))
}

/// The transaction's frame gets the whole pool, and every start forwards the whole pool its
/// caller has left. A child that succeeds hands back what it left and its net draw merges into
/// the caller's; a child that reverts or halts hands back the whole share, its draws undone, and
/// so do the draws of a child of its own that succeeded.
#[test]
fn test_the_pool_is_forwarded_whole_and_settled_at_depth() {
    let (result, handler) = at_depth().with(PoolHandler::new(Some(POOL)));

    assert!(result.is_success(), "{result:?}");
    let share = |pool| (pool, StartCharge::default(), 0);
    assert_eq!(
        handler.starts,
        [
            share(POOL),
            // PARENT drew 100.
            share(900),
            // OK drew 100 and stopped.
            share(800),
            // REVERTER and HALTER gave the whole share back.
            share(800),
            share(800),
            // OK, inside NESTER, which drew 100 first.
            share(700),
        ]
    );
    assert_eq!(
        handler.returned,
        [
            Returned::stopped(800, 100),
            Returned::ended(InstructionResult::Revert, 700, 100),
            Returned::ended(InstructionResult::InvalidFEOpcode, 700, 100),
            // OK inside NESTER.
            Returned::stopped(600, 100),
            // NESTER: its own draw and OK's, before its rollback.
            Returned::ended(InstructionResult::Revert, 600, 200),
            // PARENT: its own draw and the first OK's.
            Returned::stopped(800, 200),
        ]
    );
    assert_eq!(handler.settled, Some((800, 200, 0)));
}

/// A draw the pool cannot pay takes nothing and records the crossing; the consumer's instruction
/// halts the frame, and the record is readable on the frame's result before its caller settles
/// it, with the pool as the failed draw found it. The caller gets the whole share back.
#[test]
fn test_a_failed_draw_halts_the_frame_with_a_record_the_caller_can_read() {
    let mut parent = vec![DRAW];
    parent.extend(call(OK, 0));
    let run = Run::new(ends(parent));

    let (result, handler) = run.with(PoolHandler::new(Some(150)));

    assert!(result.is_success(), "{result:?}");
    assert_eq!(handler.starts[1], (50, StartCharge::default(), 0));
    assert_eq!(
        handler.returned,
        [
            Returned {
                crossing: true,
                ..Returned::ended(InstructionResult::OutOfGas, 50, 0)
            },
            Returned::stopped(50, 100),
        ]
    );
    assert_eq!(handler.settled, Some((50, 100, 0)));
}

/// A start charge the start hook declared rides on the frame's input, and is refunded to the
/// caller whole when the frame fails: a revert, a halt, a start the handler refused before any
/// frame ran (a transfer the caller cannot pay), one it answered without a frame (a precompile
/// out of gas), a creation that reverted and one whose caller's nonce overflowed. A frame that
/// succeeds keeps the charge.
#[test]
fn test_the_start_charge_is_refunded_when_the_frame_fails() {
    let reverting_initcode = [PUSH0, PUSH0, REVERT];
    let cases: Vec<(&str, Run, bool)> = vec![
        ("a child that stops", Run::new(call(OK, 0)), false),
        ("a child that reverts", Run::new(call(REVERTER, 0)), true),
        ("a child that halts", Run::new(call(HALTER, 0)), true),
        (
            "a transfer the caller cannot pay",
            Run::new(call(EMPTY, 1)).penniless(),
            true,
        ),
        (
            "a precompile out of gas",
            Run::new(starved_call(ECRECOVER)),
            true,
        ),
        (
            "a creation that reverts",
            Run::new(create(&reverting_initcode)),
            true,
        ),
        (
            "a creation whose caller's nonce overflowed",
            Run::new(create(&[STOP])).nonce(u64::MAX),
            true,
        ),
    ];
    for (name, run, fails) in cases {
        let mut parent = vec![DECLARE];
        parent.extend(run.parent.clone());
        let run = Run {
            parent: ends(parent),
            ..run
        };

        let (result, handler) = run.with(PoolHandler::new(Some(POOL)));

        assert!(result.is_success(), "{name}: {result:?}");
        assert_eq!(handler.starts.len(), 2, "{name}");
        assert_eq!(
            handler.starts[1],
            (POOL - DECLARED.pools.get(FIRST), DECLARED, 0),
            "{name}: the charge rides on the input"
        );
        let settled = handler.settled.expect(name);
        if fails {
            assert_eq!(settled, (POOL, 0, 0), "{name}: refunded whole");
        } else {
            let drawn = DECLARED.pools.get(FIRST) + DRAW_BYTES;
            assert_eq!(
                settled,
                (POOL - drawn, drawn as i64, DECLARED.history as i64),
                "{name}: kept"
            );
        }
    }
}

/// The transaction's frame gets the pool as the consumer left it after the body's charges, and
/// the start charge the consumer declared on its input, which is refunded whole when the frame
/// reverts or halts and kept when it succeeds: a call's frame and a creation's alike.
#[test]
fn test_the_first_frame_gets_the_pool_and_its_start_charge() {
    let cases = [
        ("stops", vec![DRAW, STOP], false),
        ("reverts", vec![DRAW, PUSH0, PUSH0, REVERT], true),
        ("halts", vec![DRAW, INVALID], true),
    ];
    let calls = cases
        .clone()
        .map(|(name, code, fails)| (name, Run::new(code), fails));
    let creations = cases.map(|(name, code, fails)| (name, Run::creation(code), fails));
    for (name, run, fails) in calls.into_iter().chain(creations) {
        let name = format!(
            "{} {name}",
            if run.creates { "a creation" } else { "a call" }
        );
        let name = name.as_str();
        let mut handler = PoolHandler::new(Some(POOL));
        handler.declare = true;

        let (result, handler) = run.with(handler);

        assert_eq!(result.is_success(), !fails, "{name}: {result:?}");
        assert_eq!(
            handler.starts,
            [(POOL - DECLARED.pools.get(FIRST), DECLARED, 0)],
            "{name}"
        );
        let settled = handler.settled.expect(name);
        if fails {
            assert_eq!(settled, (POOL, 0, 0), "{name}: refunded whole");
        } else {
            let drawn = DECLARED.pools.get(FIRST) + DRAW_BYTES;
            assert_eq!(
                settled,
                (POOL - drawn, drawn as i64, DECLARED.history as i64),
                "{name}: kept"
            );
        }
    }
}

/// A value call's child starts with the schedule's history allowance and no other frame does;
/// nothing but the consumer draws it, so a frame that charges no history returns it whole, and a
/// consumer's charge takes the allowance before the budget.
#[test]
fn test_the_history_allowance_reaches_a_value_calls_child_and_only_the_consumer_draws_it() {
    let mut parent = call(LOGGER, 1);
    parent.extend(call(LOGGER, 0));
    parent.extend(call(OK, 1));
    let run = Run::new(ends(parent)).allowance(ALLOWANCE);

    let (result, handler) = run.with(PoolHandler::new(None));

    assert!(result.is_success(), "{result:?}");
    let shares: Vec<u64> = handler.starts.iter().map(|start| start.0).collect();
    assert_eq!(shares, [u64::MAX; 4], "no pool was set");
    let allowances: Vec<u64> = handler.starts.iter().map(|start| start.2).collect();
    assert_eq!(allowances, [0, ALLOWANCE, 0, ALLOWANCE]);
    assert_eq!(
        handler.returned,
        [
            // The value call's logger: the allowance paid the whole charge.
            Returned {
                allowance: ALLOWANCE - HISTORY_COST,
                ..Returned::stopped(u64::MAX, 0)
            },
            // The plain call's logger: no allowance, the budget paid.
            Returned {
                history: HISTORY_COST as i64,
                ..Returned::stopped(u64::MAX, 0)
            },
            // A value call that charges no history keeps its allowance whole.
            Returned {
                allowance: ALLOWANCE,
                ..Returned::stopped(u64::MAX - DRAW_BYTES, DRAW_BYTES as i64)
            },
            // PARENT, with the plain call's history charge merged in and no allowance of its own.
            Returned {
                history: HISTORY_COST as i64,
                ..Returned::stopped(u64::MAX - DRAW_BYTES, DRAW_BYTES as i64)
            },
        ]
    );
    assert_eq!(
        handler.settled,
        Some((
            u64::MAX - DRAW_BYTES,
            DRAW_BYTES as i64,
            HISTORY_COST as i64
        ))
    );

    // On the default schedule no frame gets an allowance.
    let (_, handler) = Run::new(ends(call(LOGGER, 1))).with(PoolHandler::new(None));
    assert_eq!(handler.starts[1].2, 0);
    assert_eq!(handler.returned[0].history, HISTORY_COST as i64);
}

/// A consumer that sets nothing runs on an unlimited pool, which every start forwards whole, less
/// only what the test's own draws took, and every return settles back; the transaction ends as it
/// does on the mainnet handler.
#[test]
fn test_a_consumer_that_sets_nothing_sees_the_mainnet_result() {
    let run = at_depth();
    let (result, handler) = run.with(PoolHandler::new(None));

    assert_eq!(result, run.on_mainnet());
    let shares: Vec<u64> = handler.starts.iter().map(|start| start.0).collect();
    let less = |draws: u64| u64::MAX - draws * DRAW_BYTES;
    assert_eq!(
        shares,
        [less(0), less(1), less(2), less(2), less(2), less(3)],
        "the unlimited pool, less the draws so far"
    );
    assert_eq!(handler.settled, Some((less(2), 2 * DRAW_BYTES as i64, 0)));
}

/// The admission hook draws deployed code's bytes from the creating frame's pool. An admitted
/// deposit's draw is the creation's own: its caller takes its net over and the transaction's net
/// counts it, as for any draw of a frame that succeeded. A refused deposit ends the creation as a
/// revert with the draw undone, and its caller gets the whole share back.
#[test]
fn test_a_deposit_drawn_at_admission_is_held_against_the_transaction() {
    // Deploys 32 zero bytes.
    let initcode = [PUSH1, 32, PUSH0, RETURN];
    for (name, deposit, returned, settled) in [
        (
            "admitted",
            Deposit::Drawn,
            Returned::ended(InstructionResult::Return, POOL - 32, 32),
            (POOL - 32, 32, 0),
        ),
        (
            "refused",
            Deposit::DrawnAndRefused,
            Returned::ended(InstructionResult::Revert, POOL, 0),
            (POOL, 0, 0),
        ),
    ] {
        let run = Run::new(ends(create(&initcode))).deposit(deposit);

        let (result, handler) = run.with(PoolHandler::new(Some(POOL)));

        assert!(result.is_success(), "{name}: {result:?}");
        assert_eq!(
            handler.starts[1].0, POOL,
            "{name}: the whole pool forwarded"
        );
        assert_eq!(handler.returned[0], returned, "{name}");
        assert_eq!(handler.settled, Some(settled), "{name}");
    }
}

/// A precompile's answer starts with its caller's share as its pool, less what the provider drew
/// on it, so its pool and its net add up to the share, as a frame's do: the caller takes the
/// provider's draw over when the call succeeds, and gets the whole share back when it fails.
#[test]
fn test_a_precompile_answer_keeps_what_its_provider_drew() {
    const PROVIDER_DRAW: u64 = 40;
    let mut succeeding = vec![DRAW];
    succeeding.extend(call(ECRECOVER, 0));
    let mut failing = vec![DRAW];
    failing.extend(starved_call(ECRECOVER));
    // PARENT drew 100 and forwards the 900 left; the provider draws 40 of them.
    for (name, parent, returned, settled) in [
        (
            "succeeds",
            succeeding,
            Returned::ended(InstructionResult::Return, 860, 40),
            (860, 140, 0),
        ),
        (
            "fails",
            failing,
            Returned::ended(InstructionResult::PrecompileOOG, 860, 40),
            (900, 100, 0),
        ),
    ] {
        let run = Run::new(ends(parent)).precompile_draw(PROVIDER_DRAW);

        let (result, handler) = run.with(PoolHandler::new(Some(POOL)));

        assert!(result.is_success(), "{name}: {result:?}");
        assert_eq!(handler.starts[1].0, 900, "{name}");
        assert_eq!(handler.returned[0], returned, "{name}");
        assert_eq!(handler.settled, Some(settled), "{name}");
    }
}

/// A provider that draws on an answer without starting its pools at the shares draws on the
/// unlimited pools the answer was built with, so it can draw past a share. Within the share the
/// answer is adopted as any is; past it, each pool fails closed: the answer holds an empty pool,
/// a net of exactly the share and the pool's crossing, and the caller ends with exactly what it
/// held, whether the call succeeds or fails. The answer's result is the provider's.
#[test]
fn test_a_precompile_answer_that_drew_past_its_share_is_a_crossing_held_to_the_share() {
    let succeeding = call(ECRECOVER, 0);
    let failing = starved_call(ECRECOVER);
    // Every pool of 1,000 forwards 10%, 100, and PARENT keeps 900.
    for (name, parent, draw, returned, settled) in [
        (
            "within, succeeds",
            succeeding.clone(),
            40,
            Returned::ended(InstructionResult::Return, 60, 40),
            (960, 40),
        ),
        (
            "the whole share, succeeds",
            succeeding.clone(),
            100,
            Returned::ended(InstructionResult::Return, 0, 100),
            (900, 100),
        ),
        (
            "past it, succeeds",
            succeeding,
            150,
            Returned {
                crossing: true,
                ..Returned::ended(InstructionResult::Return, 0, 100)
            },
            (900, 100),
        ),
        (
            "past it, fails",
            failing,
            150,
            Returned {
                crossing: true,
                ..Returned::ended(InstructionResult::PrecompileOOG, 0, 100)
            },
            (1_000, 0),
        ),
    ] {
        let run = Run::new(ends(parent))
            .pool_percents([10; PoolId::COUNT])
            .unseeded_precompile_draw(draw);

        let (result, handler) = run.with(PoolHandler::new(Some(POOL)));

        assert!(result.is_success(), "{name}: {result:?}");
        assert_eq!(
            handler.pool_starts[1],
            PoolAmounts::ZERO
                .with(FIRST, 100)
                .with(PoolId::new(1), 100)
                .with(PoolId::new(2), 100),
            "{name}"
        );
        assert_eq!(handler.returned[0], returned, "{name}");
        assert_eq!(
            handler.settled_pools,
            Some([settled; PoolId::COUNT]),
            "{name}: every pool"
        );
    }
}

/// A precompile's answer to a value call carries the history allowance the call instruction gave
/// the start, as a frame does, and an answer to a call without value carries none.
#[test]
fn test_a_precompile_answer_to_a_value_call_carries_the_allowance() {
    let mut parent = call(ECRECOVER, 1);
    parent.extend(call(ECRECOVER, 0));
    let run = Run::new(ends(parent)).allowance(ALLOWANCE);

    let (result, handler) = run.with(PoolHandler::new(None));

    assert!(result.is_success(), "{result:?}");
    let allowances: Vec<u64> = handler.starts.iter().map(|start| start.2).collect();
    assert_eq!(allowances, [0, ALLOWANCE, 0]);
    let answered = Returned::ended(InstructionResult::Return, u64::MAX, 0);
    assert_eq!(
        handler.returned[..2],
        [
            Returned {
                allowance: ALLOWANCE,
                ..answered.clone()
            },
            answered,
        ]
    );
}

/// A start the handler refuses for the call depth gives its caller the whole share and the start
/// charge back, as any start that fails does, for a call and a creation alike. The handler's loop
/// starts the frame one past the call-stack limit, where a 1,025th nested start would be.
#[test]
fn test_a_start_refused_for_the_depth_gives_the_share_and_the_start_charge_back() {
    for (name, start) in [("a call", call(OK, 0)), ("a creation", create(&[STOP]))] {
        let mut parent = vec![DRAW, DECLARE];
        parent.extend(start);
        let mut handler = PoolHandler::new(Some(POOL));
        handler.too_deep = true;

        let (result, handler) = Run::new(ends(parent)).with(handler);

        assert!(result.is_success(), "{name}: {result:?}");
        let share = POOL - DRAW_BYTES - DECLARED.pools.get(FIRST);
        assert_eq!(handler.starts[1], (share, DECLARED, 0), "{name}");
        assert_eq!(
            handler.returned[0],
            Returned::ended(InstructionResult::CallTooDeep, share, 0),
            "{name}"
        );
        assert_eq!(
            handler.settled,
            Some((POOL - DRAW_BYTES, DRAW_BYTES as i64, 0)),
            "{name}: the start charge refunded, PARENT's own draw kept"
        );
    }
}

/// With [`FIRST`]'s `pool_kept_percent` at 2, every start forwards 98% of its caller's pool, rounded
/// down, and the caller keeps the rest; what a child hands back adds to what its caller kept, so
/// the transaction ends with the figures the whole-pool forward gives.
#[test]
fn test_a_share_below_the_whole_pool_is_forwarded_and_settled_at_depth() {
    let (result, handler) = at_depth()
        .forward_percent(98)
        .with(PoolHandler::new(Some(POOL)));

    assert!(result.is_success(), "{result:?}");
    let share = |pool| (pool, StartCharge::default(), 0);
    assert_eq!(
        handler.starts,
        [
            // The transaction's frame takes the whole pool.
            share(POOL),
            // PARENT drew 100: 98% of 900, PARENT keeps 18.
            share(882),
            // OK left 782, PARENT holds 800 again: 98% of 800, PARENT keeps 16.
            share(784),
            share(784),
            share(784),
            // NESTER drew 100 of its 784: 98% of 684 is 670.32, NESTER keeps 14.
            share(670),
        ]
    );
    assert_eq!(
        handler.returned,
        [
            Returned::stopped(782, 100),
            Returned::ended(InstructionResult::Revert, 684, 100),
            Returned::ended(InstructionResult::InvalidFEOpcode, 684, 100),
            // OK inside NESTER.
            Returned::stopped(570, 100),
            // NESTER: the 14 it kept and the 570 OK left, before its rollback.
            Returned::ended(InstructionResult::Revert, 584, 200),
            // PARENT: 16 kept and the whole 784 NESTER gave back.
            Returned::stopped(800, 200),
        ]
    );
    assert_eq!(handler.settled, Some((800, 200, 0)));
}

/// Each pool is forwarded by its own share and settles alone: with the pools' percentages at 98,
/// 50 and 100, every start forwards that much of what its caller has left of each pool, rounded
/// down, and the caller keeps the rest of each; what a child hands back of each pool adds to what
/// its caller kept, so every pool ends where the whole-pool forward ends it.
#[test]
fn test_each_pool_is_forwarded_by_its_own_share_and_settled_at_depth() {
    let (result, handler) = at_depth()
        .pool_percents([98, 50, 100])
        .with(PoolHandler::new(Some(POOL)));

    assert!(result.is_success(), "{result:?}");
    // PARENT drew 100 of each pool before each start; OK drew 100 and handed back the rest, so
    // PARENT holds 800 of each for the later starts. NESTER drew 100 of each of its shares (784,
    // 400, 800) before calling OK.
    let each = |shares: [u64; PoolId::COUNT]| {
        PoolAmounts::ZERO
            .with(PoolId::new(0), shares[0])
            .with(PoolId::new(1), shares[1])
            .with(PoolId::new(2), shares[2])
    };
    assert_eq!(
        handler.pool_starts,
        [
            each([POOL; 3]),
            // 98%, 50% and 100% of 900.
            each([882, 450, 900]),
            // Of 800.
            each([784, 400, 800]),
            each([784, 400, 800]),
            each([784, 400, 800]),
            // Of 684, 300 and 700: 98% of 684 is 670.32.
            each([670, 150, 700]),
        ]
    );
    assert_eq!(handler.settled_pools, Some([(800, 200); 3]));
}
