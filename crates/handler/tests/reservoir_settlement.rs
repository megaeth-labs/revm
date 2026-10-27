//! A child frame's EIP-8037 reservoir, as the mainnet handler forwards and settles it, and as a
//! consumer that keeps part of it for the caller settles it through the start lifecycle.
//!
//! A frame start copies its caller's reservoir into the child's input, and the caller keeps it
//! while the child runs; the child's return replaces the caller's reservoir with what the child
//! left, once that has unwound the caller's outstanding spill. That is upstream's forward and
//! settlement, and the default.
//!
//! A consumer that keeps part of the reservoir for the caller takes it in the frame-start hook,
//! names the child's reservoir on the start (`FrameStart::reservoir`), and adds back the reservoir
//! the caller held (`FrameSettle::held_reservoir`) when revm settles the child. The tests here keep
//! 2% that way, through [`KeepingContext`], and get the figures a forward of 98% that adds the
//! child's return to the part the caller kept gives.
//!
//! The transactions run on Amsterdam with a gas limit [`RESERVOIR`] above the regular cap, so they
//! start with a reservoir. The frames charge state gas through an instruction of the test's own,
//! `CHARGE`, which stands in for a write's state charge, and through `SSTORE`. The handler is the
//! mainnet one, observed through overrides that leave its behaviour alone: each frame's reservoir
//! as it starts, and as it returns before its caller settles it.

use bytecode::opcode::{
    CALL, CREATE, DELEGATECALL, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH2, PUSH20, PUSH3, REVERT,
    SSTORE, STOP,
};
use context::{
    result::{EVMError, ExecutionResult},
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Journal, LocalContext,
    TxEnv,
};
use context_interface::{
    cfg::{gas::InitialAndFloorGas, GasId, GasParams, StateGasSite},
    context::{SStoreResult, SelfDestructResult, StateLoad},
    host::{FrameSettle, FrameSplit, FrameStart, LifecycleEnd, LoadError},
    journaled_state::AccountInfoLoad,
    Cfg, Database, Host,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, FrameInput, GasTracker,
    Instruction, InstructionContext, InstructionExecResult, InstructionResult,
};
use primitives::{
    address, constants::CALL_STACK_LIMIT, hardfork::SpecId, Address, Bytes, Log, StorageKey,
    StorageValue, TxKind, B256, U256,
};
use revm_handler::{
    instructions::EthInstructions, system_call::SystemCallTx, EthFrame, EthPrecompiles, EvmTr,
    FrameResult, Handler, ItemOrResult, MainContext, MainnetContext, MainnetHandler,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = KeepingContext;
type TestEvm =
    Evm<TestContext, (), EthInstructions<EthInterpreter, TestContext>, EthPrecompiles, EthFrame>;
type TestError = EVMError<DbError>;
type Mainnet = MainnetHandler<TestEvm, TestError, EthFrame<EthInterpreter>>;

const SPEC: SpecId = SpecId::AMSTERDAM;
const GAS_LIMIT: u64 = 3_000_000;
/// The reservoir a transaction starts with: the regular cap is set this far below
/// [`GAS_LIMIT`], and no transaction here pays state gas in its intrinsic cost.
const RESERVOIR: u64 = 1_000_000;
/// What `CHARGE` charges as state gas.
const CHARGED: u64 = 1_000;
/// The share of a caller's reservoir a consumer that keeps part of it forwards.
const PERCENT: u64 = 98;
/// State gas of a new storage slot on Amsterdam's schedule.
const SLOT: u64 = 97_920;

/// The account the transaction calls.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Charges and stops.
const OK: Address = address!("0x0000000000000000000000000000000000000c00");
/// Charges and reverts.
const REVERTER: Address = address!("0x000000000000000000000000000000000000c0fd");
/// Charges and halts.
const HALTER: Address = address!("0x000000000000000000000000000000000000c0fe");
/// Charges, calls [`OK`] and reverts.
const NESTER: Address = address!("0x000000000000000000000000000000000000c0e5");
/// Charges a thousand times what `CHARGE` does, past any share here.
const SPENDER: Address = address!("0x000000000000000000000000000000000000c5e0");
/// Clears its caller's slot 0 and stops, run by `DELEGATECALL` on the caller's storage.
const RESTORER: Address = address!("0x000000000000000000000000000000000000c0e0");
/// Clears its caller's slot 0 and reverts.
const RESTORER_REVERTS: Address = address!("0x000000000000000000000000000000000000c0e1");
/// The `ECRECOVER` precompile.
const ECRECOVER: Address = address!("0x0000000000000000000000000000000000000001");

/// The test's instruction, on an opcode the EVM does not assign.
const CHARGE: u8 = 0x0c;

/// Charges [`CHARGED`] of state gas on the frame, reservoir first, as a write's state charge
/// does; a charge the frame cannot pay is its out-of-gas.
const fn charge(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    if context.interpreter.gas.record_state_cost(CHARGED) {
        Ok(())
    } else {
        Err(InstructionResult::OutOfGas)
    }
}

/// A frame's figures as it returned, before its caller settled it: its result, its reservoir and
/// its net state gas.
type Returned = (InstructionResult, u64, i64);

/// The mainnet context, with the start lifecycle's hooks of a consumer that keeps part of each
/// caller's reservoir for the caller, when told to ([`KeepingContext::percent`]).
///
/// The frame-start hook takes `100 - percent` per cent of the caller's reservoir, rounded down,
/// for the caller, and names the rest as the child's reservoir; the settlement hook adds the
/// reservoir the caller held back to it once revm has settled a child into it. The transaction's
/// frame is given the whole reservoir, so nothing is added back to the transaction's gas. Each
/// settlement's depth and held reservoir is recorded, told to keep or not.
struct KeepingContext {
    inner: MainnetContext<Db>,
    /// The per cent of a caller's reservoir a child receives, if the consumer keeps the rest.
    percent: Option<u64>,
    /// Every settlement's depth and `held_reservoir`, in the order revm settled them.
    held: Vec<(usize, u64)>,
}

impl ContextTr for KeepingContext {
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

impl ContextSetters for KeepingContext {
    fn set_tx(&mut self, tx: Self::Tx) {
        self.inner.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.inner.set_block(block);
    }
}

impl Host for KeepingContext {
    fn frame_starting(&mut self, start: &mut FrameStart, gas: &mut GasTracker) -> bool {
        if let Some(percent) = self.percent {
            let reservoir = gas.reservoir();
            let kept = reservoir * (100 - percent) / 100;
            start.reservoir = Some(reservoir - kept);
            gas.set_reservoir(kept);
        }
        true
    }

    fn frame_split(&mut self, split: &FrameSplit) {
        self.inner.frame_split(split);
    }

    fn frame_settle(&mut self, settle: &FrameSettle, gas: &mut GasTracker) {
        self.held.push((settle.depth, settle.held_reservoir));
        if self.percent.is_some() && settle.depth > 0 {
            gas.set_reservoir(gas.reservoir() + settle.held_reservoir);
        }
        self.inner.frame_settle(settle, gas);
    }

    fn frame_lifecycle_end(&mut self, end: LifecycleEnd) {
        self.inner.frame_lifecycle_end(end);
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

const fn returned(result: &FrameResult) -> Returned {
    let gas = result.gas();
    (
        result.instruction_result(),
        gas.reservoir(),
        gas.state_gas_spent(),
    )
}

fn reservoir_of(input: &FrameInput) -> u64 {
    match input {
        FrameInput::Call(inputs) => inputs.reservoir,
        FrameInput::Create(inputs) => inputs.reservoir(),
        FrameInput::Empty => unreachable!(),
    }
}

/// The mainnet handler, recording every frame's reservoir as it starts and its figures as it
/// returns, and the transaction's gas once the first frame settled into it.
struct ShareHandler {
    mainnet: Mainnet,
    /// Whether every frame below the transaction's starts one past the call-stack limit, where a
    /// 1,025th nested start would.
    too_deep: bool,
    /// A reservoir to give the transaction's gas, as a consumer that splits a system call's gas
    /// does; mainnet gives a system call none.
    system_reservoir: Option<u64>,
    starts: Vec<u64>,
    returned: Vec<Returned>,
    /// The transaction's reservoir and net state gas after the first frame settled into it.
    settled: Option<(u64, i64)>,
    /// The reservoir of the last gas `tx_gas` built.
    tx_reservoir: core::cell::Cell<Option<u64>>,
    /// The transaction's reservoir once the first frame's input was built.
    after_first_input: Option<u64>,
    /// Every settlement's depth and held reservoir, once the transaction's start settled.
    held: Vec<(usize, u64)>,
}

impl ShareHandler {
    fn new() -> Self {
        Self {
            mainnet: Mainnet::default(),
            too_deep: false,
            system_reservoir: None,
            starts: Vec::new(),
            returned: Vec::new(),
            settled: None,
            tx_reservoir: core::cell::Cell::new(None),
            after_first_input: None,
            held: Vec::new(),
        }
    }
}

impl Handler for ShareHandler {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = context::result::HaltReason;

    fn tx_gas(&self, evm: &mut TestEvm, init_and_floor_gas: &InitialAndFloorGas) -> GasTracker {
        let mut gas = self.mainnet.tx_gas(evm, init_and_floor_gas);
        if let Some(reservoir) = self.system_reservoir {
            gas.set_remaining(gas.remaining() - reservoir);
            gas.set_reservoir(reservoir);
        }
        self.tx_reservoir.set(Some(gas.reservoir()));
        gas
    }

    fn first_frame_input(
        &mut self,
        evm: &mut TestEvm,
        gas: &mut GasTracker,
    ) -> Result<Option<FrameInit>, TestError> {
        let input = self.mainnet.first_frame_input(evm, gas)?;
        self.after_first_input = Some(gas.reservoir());
        Ok(input)
    }

    /// The default loop, recording each frame's reservoir as it starts and its figures as it
    /// returns, and starting the frames below the transaction's past the call-stack limit if told
    /// to.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        self.starts
            .push(reservoir_of(&first_frame_input.frame_input));
        if let ItemOrResult::Result(result) = evm.frame_init(first_frame_input)? {
            self.returned.push(returned(&result));
            return Ok(result);
        }
        loop {
            let result = match evm.frame_run()? {
                ItemOrResult::Item(mut init) => {
                    self.starts.push(reservoir_of(&init.frame_input));
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
            self.returned.push(returned(&result));
            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }

    /// The default, recording the transaction's gas as the first frame left it.
    fn last_frame_result(
        &mut self,
        evm: &mut TestEvm,
        frame_result: &mut FrameResult,
        gas: &mut GasTracker,
    ) -> Result<(), TestError> {
        self.mainnet.last_frame_result(evm, frame_result, gas)?;
        self.settled = Some((gas.reservoir(), gas.state_gas_spent()));
        self.held = evm.ctx().held.clone();
        Ok(())
    }
}

/// `op(gas, to, value, 0, 0, 0, 0)` for `CALL`, `op(gas, to, 0, 0, 0, 0)` for `DELEGATECALL`,
/// dropping the success flag.
fn call_with(op: u8, to: Address, value: u8, gas: u32) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0];
    if op == CALL {
        code.extend_from_slice(&[PUSH1, value]);
    }
    code.push(PUSH20);
    code.extend_from_slice(to.as_slice());
    code.push(PUSH3);
    code.extend_from_slice(&gas.to_be_bytes()[1..]);
    code.extend_from_slice(&[op, POP]);
    code
}

/// `CALL(100_000, to, 0, 0, 0, 0, 0)`, dropping the success flag.
fn call(to: Address) -> Vec<u8> {
    call_with(CALL, to, 0, 100_000)
}

fn ends(mut code: Vec<u8>) -> Vec<u8> {
    code.push(STOP);
    code
}

/// A transaction that calls [`PARENT`], which holds `parent`.
struct Run {
    parent: Vec<u8>,
    parent_balance: u64,
    /// The per cent of the reservoir a child receives, if the consumer keeps the rest for the
    /// caller ([`KeepingContext`]); by default it receives the whole reservoir.
    percent: Option<u64>,
    /// The transaction, if not a call to [`PARENT`] with [`GAS_LIMIT`].
    tx: Option<TxEnv>,
    /// The regular cap, if not [`RESERVOIR`] below [`GAS_LIMIT`].
    cap: Option<u64>,
    /// An account with code at [`PARENT`]'s first created address, where a `CREATE` collides.
    occupied: bool,
}

impl Run {
    const fn new(parent: Vec<u8>) -> Self {
        Self {
            parent,
            parent_balance: 10,
            percent: None,
            tx: None,
            cap: None,
            occupied: false,
        }
    }

    const fn percent(mut self, percent: u64) -> Self {
        self.percent = Some(percent);
        self
    }

    /// A transaction that creates a contract from `initcode`.
    fn creation(initcode: Vec<u8>) -> Self {
        let mut run = Self::new(Vec::new());
        run.tx = Some(
            TxEnv::builder_for_bench()
                .kind(TxKind::Create)
                .data(Bytes::from(initcode))
                .gas_price(0)
                .gas_limit(GAS_LIMIT)
                .build_fill(),
        );
        run
    }

    const fn occupied(mut self) -> Self {
        self.occupied = true;
        self
    }

    const fn penniless(mut self) -> Self {
        self.parent_balance = 0;
        self
    }

    fn db(&self) -> Db {
        let mut db = Db::default();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo::from_balance(U256::from(10u128.pow(21))),
        );
        let mut nester = vec![CHARGE];
        nester.extend(call(OK));
        nester.extend_from_slice(&[PUSH0, PUSH0, REVERT]);
        let accounts: [(Address, Vec<u8>, u64); 8] = [
            (PARENT, self.parent.clone(), self.parent_balance),
            (OK, vec![CHARGE, STOP], 0),
            (REVERTER, vec![CHARGE, PUSH0, PUSH0, REVERT], 0),
            (HALTER, vec![CHARGE, INVALID], 0),
            (NESTER, nester, 0),
            (
                SPENDER,
                [CHARGE; 1_000].into_iter().chain([STOP]).collect(),
                0,
            ),
            (RESTORER, vec![PUSH0, PUSH0, SSTORE, STOP], 0),
            (
                RESTORER_REVERTS,
                vec![PUSH0, PUSH0, SSTORE, PUSH0, PUSH0, REVERT],
                0,
            ),
        ];
        for (address, code, balance) in accounts {
            let info = AccountInfo::from_balance(U256::from(balance))
                .with_nonce(1)
                .with_code(Bytecode::new_raw(Bytes::from(code)));
            db.insert_account_info(address, info);
        }
        if self.occupied {
            db.insert_account_info(
                PARENT.create(1),
                AccountInfo::default()
                    .with_nonce(1)
                    .with_code(Bytecode::new_raw(Bytes::from_static(&[STOP]))),
            );
        }
        db
    }

    fn evm(&self) -> TestEvm {
        let percent = self.percent;
        let cap = self.cap.unwrap_or(GAS_LIMIT - RESERVOIR);
        let ctx = Context::mainnet()
            .with_db(self.db())
            .modify_cfg_chained(|cfg| {
                cfg.set_spec_and_mainnet_gas_params(SPEC);
                cfg.tx_gas_limit_cap = Some(cap);
            });
        let ctx = KeepingContext {
            inner: ctx,
            percent,
            held: Vec::new(),
        };
        let mut instructions = EthInstructions::new_mainnet_with_spec(SPEC);
        instructions.insert_instruction(CHARGE, Instruction::new(charge), 0);
        let mut evm: TestEvm = Evm::new(ctx, instructions, EthPrecompiles::new(SPEC));
        evm.ctx.set_tx(self.tx.clone().unwrap_or_else(|| {
            TxEnv::builder_for_bench()
                .kind(TxKind::Call(PARENT))
                .gas_price(0)
                .gas_limit(GAS_LIMIT)
                .build_fill()
        }));
        evm
    }

    /// Runs the transaction with `handler` and returns its result and the handler, with what it
    /// recorded.
    fn with(&self, mut handler: ShareHandler) -> (ExecutionResult, ShareHandler) {
        let result = handler.run(&mut self.evm()).expect("the transaction runs");
        (result, handler)
    }

    fn run(&self) -> (ExecutionResult, ShareHandler) {
        self.with(ShareHandler::new())
    }
}

/// [`PARENT`] charges, then calls a child that stops, one that reverts, one that halts and one
/// that calls a child of its own and reverts, each after a charge of its own.
fn at_depth() -> Run {
    let mut parent = vec![CHARGE];
    for child in [OK, REVERTER, HALTER, NESTER] {
        parent.extend(call(child));
    }
    Run::new(ends(parent))
}

/// The share at 98: `reservoir` less the caller's 2%, rounded down.
const fn share(reservoir: u64) -> u64 {
    reservoir - reservoir * (100 - PERCENT) / 100
}

/// A consumer that keeps 2% of each caller's reservoir, rounded down, gives each start the rest;
/// revm's settlement replaces the caller's reservoir with what the child hands back, and the
/// consumer adds what the caller held, the part it kept: what the child left when it succeeded,
/// the whole share when it reverted or halted, a successful grandchild's charge included, joins
/// the kept part. The transaction's frame is given the whole reservoir, which the transaction's gas
/// holds as its copy, so nothing is added back to it. No frame here spends past its share, so the
/// transaction ends with the figures the whole forward gives, and the same result.
#[test]
fn test_a_consumer_that_keeps_part_of_the_reservoir_adds_back_what_the_caller_held() {
    let (result, handler) = at_depth().percent(PERCENT).run();

    assert!(result.is_success(), "{result:?}");
    // PARENT charged 1,000 of its 1,000,000 before each start below.
    let after_parent = RESERVOIR - CHARGED;
    assert_eq!(share(after_parent), 979_020);
    // OK returns 978,020, so PARENT holds 19,980 + 978,020 = 998,000 for each later start.
    let before_each = after_parent - CHARGED;
    assert_eq!(share(before_each), 978_040);
    // NESTER charged 1,000 of its 978,040 before calling OK: it keeps 2% of 977,040 rounded
    // down, 19,540, where 98% rounded down would have forwarded 957,499.
    let nester_after = share(before_each) - CHARGED;
    assert_eq!(share(nester_after), 957_500);
    assert_eq!(
        handler.starts,
        [
            RESERVOIR,
            share(after_parent),
            share(before_each),
            share(before_each),
            share(before_each),
            share(nester_after),
        ]
    );
    let charged = CHARGED as i64;
    assert_eq!(
        handler.returned,
        [
            (
                InstructionResult::Stop,
                share(after_parent) - CHARGED,
                charged
            ),
            (
                InstructionResult::Revert,
                share(before_each) - CHARGED,
                charged
            ),
            (
                InstructionResult::InvalidFEOpcode,
                share(before_each) - CHARGED,
                charged
            ),
            // OK under NESTER: 957,500 less its charge.
            (
                InstructionResult::Stop,
                share(nester_after) - CHARGED,
                charged
            ),
            // NESTER kept 19,540 and got back 956,500: 976,040, two charges net.
            (InstructionResult::Revert, 976_040, 2 * charged),
            (
                InstructionResult::Stop,
                RESERVOIR - 2 * CHARGED,
                2 * charged
            ),
        ]
    );
    assert_eq!(
        handler.settled,
        Some((RESERVOIR - 2 * CHARGED, 2 * charged))
    );
    // What each caller held when revm settled its child: the 2% it kept, of 999,000, 998,000 and
    // 977,040; the transaction's gas held the copy of the whole reservoir it gave its frame.
    assert_eq!(
        handler.held,
        [
            (1, 19_980),
            (1, 19_960),
            (1, 19_960),
            (2, 19_540),
            (1, 19_960),
            (0, RESERVOIR),
        ]
    );

    let (whole, whole_handler) = at_depth().run();
    assert_eq!(result, whole, "the shares change no result here");
    assert_eq!(whole_handler.settled, handler.settled);
}

/// The default forward is upstream's: every start copies all its caller has into the child's
/// input, the caller keeps its reservoir while the child runs, and the child's return replaces it.
/// The transaction's gas keeps the reservoir it gives its frame, which its frame's return replaces.
#[test]
fn test_every_start_copies_the_whole_reservoir_by_default() {
    let (result, handler) = at_depth().run();

    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        handler.after_first_input,
        Some(RESERVOIR),
        "the transaction's gas keeps its reservoir"
    );
    let before_each = RESERVOIR - 2 * CHARGED;
    assert_eq!(
        handler.starts,
        [
            RESERVOIR,
            RESERVOIR - CHARGED,
            before_each,
            before_each,
            before_each,
            before_each - CHARGED,
        ]
    );
    let charged = CHARGED as i64;
    // NESTER and its OK each charged once on what NESTER started with.
    assert_eq!(
        handler.returned[4],
        (
            InstructionResult::Revert,
            before_each - 2 * CHARGED,
            2 * charged
        )
    );
    assert_eq!(handler.settled, Some((before_each, 2 * charged)));
    // Each caller held the reservoir it copied to its child: PARENT 999,000 at its first call and
    // 998,000, OK's return, at the others; NESTER 997,000; the transaction's gas the whole.
    assert_eq!(
        handler.held,
        [
            (1, RESERVOIR - CHARGED),
            (1, before_each),
            (1, before_each),
            (2, before_each - CHARGED),
            (1, before_each),
            (0, RESERVOIR),
        ]
    );
}

/// A child whose charges pass its share spills onto its regular gas, as a frame whose reservoir
/// is empty does, though its caller holds the 2% the consumer kept: the share is all the child
/// has. On
/// return the caller holds what it kept beside the child's spill; when the caller's own frame
/// returns, its reservoir unwinds that spill first, as any returned reservoir does (last in, first
/// out), so the transaction ends as the whole forward ends it.
#[test]
fn test_a_child_spends_past_its_share_onto_its_regular_gas() {
    let parent = ends(call(SPENDER));
    let (result, handler) = Run::new(parent.clone()).percent(PERCENT).run();
    let (whole, whole_handler) = Run::new(parent).run();

    assert!(
        result.is_success() && whole.is_success(),
        "{result:?} {whole:?}"
    );
    let spent = (1_000 * CHARGED) as i64;
    // The share is 980,000 of 1,000,000: 20,000 of the million charged spill.
    assert_eq!(handler.starts, [RESERVOIR, share(RESERVOIR)]);
    assert_eq!(
        handler.returned,
        [
            (InstructionResult::Stop, 0, spent),
            (InstructionResult::Stop, RESERVOIR - share(RESERVOIR), spent),
        ],
        "the share is spent and the rest spilled; PARENT holds what it kept"
    );
    // With the whole reservoir forwarded, the million fits it exactly and nothing spills.
    assert_eq!(
        whole_handler.returned,
        [
            (InstructionResult::Stop, 0, spent),
            (InstructionResult::Stop, 0, spent)
        ]
    );
    assert_eq!(handler.settled, Some((0, spent)));
    assert_eq!(whole_handler.settled, handler.settled);
    assert_eq!(result, whole);
}

/// The share is what the child may draw from the reservoir while it runs: a child that needs the
/// whole reservoir and has too little regular gas to pay past its share runs out of gas at 98%,
/// and its failure gives the share back, where the whole forward lets it finish.
#[test]
fn test_a_child_that_cannot_pay_past_its_share_runs_out_of_gas() {
    let parent = ends(call_with(CALL, SPENDER, 0, 10_000));
    let (result, handler) = Run::new(parent.clone()).percent(PERCENT).run();
    let (whole, whole_handler) = Run::new(parent).run();

    assert!(
        result.is_success() && whole.is_success(),
        "{result:?} {whole:?}"
    );
    // 980 charges spend the share, ten more spill its 10,000 of regular gas, the 991st fails.
    assert_eq!(
        handler.returned[0],
        (
            InstructionResult::OutOfGas,
            0,
            (991 * CHARGED - CHARGED) as i64
        )
    );
    assert_eq!(
        handler.settled,
        Some((RESERVOIR, 0)),
        "nothing charged in the end"
    );
    let spent = (1_000 * CHARGED) as i64;
    assert_eq!(
        whole_handler.returned[0],
        (InstructionResult::Stop, 0, spent)
    );
    assert_eq!(whole_handler.settled, Some((0, spent)));
}

/// A child that clears a slot its caller created gets the slot's state gas back as an EIP-8037
/// restoration, into its own reservoir; when it succeeds, the caller gets it back with the rest of
/// the share and the part the consumer kept, and is back where it was before it created the slot. When it reverts,
/// the restoration is undone with the rest of its frame and the caller gets the share back.
#[test]
fn test_a_restoration_in_the_child_comes_back_to_its_caller() {
    for (restorer, kept) in [(RESTORER, true), (RESTORER_REVERTS, false)] {
        let mut parent = vec![PUSH1, 1, PUSH0, SSTORE];
        parent.extend(call_with(DELEGATECALL, restorer, 0, 100_000));
        let (result, handler) = Run::new(ends(parent)).percent(PERCENT).run();

        assert!(result.is_success(), "{restorer}: {result:?}");
        let created = RESERVOIR - SLOT;
        assert_eq!(handler.starts, [RESERVOIR, share(created)], "{restorer}");
        if kept {
            assert_eq!(
                handler.returned[0],
                (
                    InstructionResult::Stop,
                    share(created) + SLOT,
                    -(SLOT as i64)
                ),
                "the restoration lands in the child's reservoir"
            );
            assert_eq!(
                handler.settled,
                Some((RESERVOIR, 0)),
                "nothing charged in all"
            );
        } else {
            assert_eq!(
                handler.returned[0],
                (
                    InstructionResult::Revert,
                    share(created) + SLOT,
                    -(SLOT as i64)
                ),
            );
            assert_eq!(
                handler.settled,
                Some((created, SLOT as i64)),
                "the slot stays, and its charge"
            );
        }
    }
}

/// A precompile's answer carries the share it was given, which comes back whole; so does a start
/// the handler refuses before any frame: a transfer the caller cannot pay, and a start one past
/// the call-stack limit.
#[test]
fn test_an_answer_without_a_frame_hands_the_share_back() {
    let mut precompile = vec![CHARGE];
    precompile.extend(call(ECRECOVER));
    let mut transfer = vec![CHARGE];
    transfer.extend(call_with(CALL, OK, 1, 100_000));
    let mut deep = vec![CHARGE];
    deep.extend(call(OK));
    for (name, run, too_deep, answer) in [
        (
            "a precompile",
            Run::new(ends(precompile)),
            false,
            InstructionResult::Return,
        ),
        (
            "an unpaid transfer",
            Run::new(ends(transfer)).penniless(),
            false,
            InstructionResult::OutOfFunds,
        ),
        (
            "too deep",
            Run::new(ends(deep)),
            true,
            InstructionResult::CallTooDeep,
        ),
    ] {
        let mut handler = ShareHandler::new();
        handler.too_deep = too_deep;
        let (result, handler) = run.percent(PERCENT).with(handler);

        assert!(result.is_success(), "{name}: {result:?}");
        let before = RESERVOIR - CHARGED;
        assert_eq!(handler.starts, [RESERVOIR, share(before)], "{name}");
        assert_eq!(
            handler.returned[0],
            (answer, share(before), 0),
            "{name}: the answer carries the share"
        );
        assert_eq!(
            handler.settled,
            Some((before, CHARGED as i64)),
            "{name}: PARENT's reservoir whole again"
        );
    }
}

/// A system call's frame takes the whole reservoir its gas holds, a consumer's split of the
/// system call's gas, and its children their shares, as a transaction's do.
#[test]
fn test_a_system_call_forwards_and_settles_its_reservoir() {
    for percent in [None, Some(PERCENT)] {
        let mut run = Run::new(ends(call(OK)));
        run.percent = percent;
        let mut evm = run.evm();
        evm.ctx.set_tx(TxEnv::new_system_tx(PARENT, Bytes::new()));
        let mut handler = ShareHandler::new();
        handler.system_reservoir = Some(RESERVOIR);

        let result = handler
            .run_system_call(&mut evm)
            .expect("the system call runs");

        assert!(result.is_success(), "{percent:?}: {result:?}");
        let child = if percent.is_some() {
            share(RESERVOIR)
        } else {
            RESERVOIR
        };
        assert_eq!(handler.starts, [RESERVOIR, child], "{percent:?}");
        assert_eq!(
            handler.returned[0],
            (InstructionResult::Stop, child - CHARGED, CHARGED as i64),
            "{percent:?}"
        );
        assert_eq!(
            handler.settled,
            Some((RESERVOIR - CHARGED, CHARGED as i64)),
            "{percent:?}"
        );
    }
}

/// `CREATE` of the two bytes `CHARGE STOP`, endowed with nothing, dropping the address.
fn create_charging() -> Vec<u8> {
    vec![
        PUSH2, CHARGE, STOP, PUSH0, MSTORE, PUSH1, 2, PUSH1, 30, PUSH0, CREATE, POP,
    ]
}

/// A creation's frame gets its share like a call's, after the creation instruction's own state
/// charge for the account it creates, and settles it back; a creation that lands on an occupied
/// address fails before any frame and hands the share back whole.
#[test]
fn test_a_creation_forwards_its_share_and_a_collision_hands_it_back() {
    let account = Run::new(Vec::new())
        .evm()
        .ctx
        .cfg()
        .gas_params()
        .get(GasId::create_state_gas());
    let mut parent = vec![CHARGE];
    parent.extend(create_charging());
    let after_parent = RESERVOIR - CHARGED;

    let (result, handler) = Run::new(ends(parent.clone())).percent(PERCENT).run();
    assert!(result.is_success(), "{result:?}");
    let before = after_parent - account;
    assert_eq!(handler.starts, [RESERVOIR, share(before)]);
    assert_eq!(
        handler.returned[0],
        (
            InstructionResult::Return,
            share(before) - CHARGED,
            CHARGED as i64
        ),
        "the creation's own charge is on its frame"
    );
    assert_eq!(
        handler.settled,
        Some((before - CHARGED, (2 * CHARGED + account) as i64))
    );

    let (result, handler) = Run::new(ends(parent)).percent(PERCENT).occupied().run();
    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        handler.starts,
        [RESERVOIR, share(after_parent)],
        "no charge for an occupied address"
    );
    assert_eq!(
        handler.returned[0],
        (InstructionResult::CreateCollision, share(after_parent), 0)
    );
    assert_eq!(handler.settled, Some((after_parent, CHARGED as i64)));
}

/// A creation transaction's frame is given the whole reservoir the transaction has left, which the
/// transaction's gas keeps as its copy, whether the consumer keeps part of a caller's reservoir or
/// not; the frame's return replaces it.
#[test]
fn test_a_creation_transaction_is_given_the_whole_reservoir() {
    for percent in [None, Some(PERCENT)] {
        let mut run = Run::creation(vec![CHARGE, STOP]);
        run.percent = percent;
        let (result, handler) = run.run();

        assert!(result.is_success(), "{percent:?}: {result:?}");
        let first = handler.starts[0];
        assert!(first > 0, "{percent:?}");
        assert_eq!(
            handler.after_first_input,
            Some(first),
            "{percent:?}: the transaction's gas keeps its copy"
        );
        assert_eq!(
            handler.settled.map(|(reservoir, _)| reservoir),
            Some(first - CHARGED)
        );
    }
}

/// A transaction whose EIP-2780 runtime phase runs out of gas skips execution and is settled
/// from a stand-in for its first frame: all regular gas consumed and the reservoir returned once,
/// the stand-in's replacing the copy the transaction's gas kept.
#[test]
fn test_a_runtime_out_of_gas_returns_the_reservoir_once() {
    const LIMIT: u64 = 130_000;
    const CAP: u64 = 30_000;
    for percent in [None, Some(PERCENT)] {
        // A value transfer to an account that does not exist, whose new-account state gas the
        // reservoir and the regular gas left together cannot pay.
        let mut run = Run::new(Vec::new());
        run.percent = percent;
        run.cap = Some(CAP);
        run.tx = Some(
            TxEnv::builder_for_bench()
                .kind(TxKind::Call(address!(
                    "0x00000000000000000000000000000000000e3970"
                )))
                .value(U256::from(1))
                .gas_price(0)
                .gas_limit(LIMIT)
                .build_fill(),
        );
        let (result, handler) = run.run();

        assert!(
            matches!(result, ExecutionResult::Halt { .. }),
            "{percent:?}: {result:?}"
        );
        let reservoir = handler.tx_reservoir.get().expect("tx_gas ran");
        assert!(reservoir > 0, "{percent:?}");
        assert_eq!(handler.settled, Some((reservoir, 0)), "{percent:?}");
        assert_eq!(
            result.gas().total_gas_spent(),
            LIMIT - reservoir,
            "{percent:?}: every regular gas spent, the reservoir returned"
        );
    }
}

/// The same for a creation transaction, whose runtime phase charges the account it would create:
/// the stand-in carries the whole reservoir, which settles back once.
#[test]
fn test_a_creation_transactions_runtime_out_of_gas_returns_the_reservoir_once() {
    const LIMIT: u64 = 160_000;
    const CAP: u64 = 60_000;
    for percent in [None, Some(PERCENT)] {
        // A creation whose account's state gas the reservoir and the regular gas left together
        // cannot pay.
        let mut run = Run::new(Vec::new());
        run.percent = percent;
        run.cap = Some(CAP);
        run.tx = Some(
            TxEnv::builder_for_bench()
                .kind(TxKind::Create)
                .gas_price(0)
                .gas_limit(LIMIT)
                .build_fill(),
        );
        let (result, handler) = run.with(ShareHandler::new());

        assert!(
            matches!(result, ExecutionResult::Halt { .. }),
            "{percent:?}: {result:?}"
        );
        let reservoir = handler.tx_reservoir.get().expect("tx_gas ran");
        assert!(reservoir > 0, "{percent:?}");
        assert_eq!(handler.settled, Some((reservoir, 0)), "{percent:?}");
        assert_eq!(
            result.gas().total_gas_spent(),
            LIMIT - reservoir,
            "{percent:?}: every regular gas spent, the reservoir returned"
        );
    }
}

/// A runner that settles a first frame into gas that still holds the reservoir the frame was given,
/// as the transaction's gas does, gets the frame's reservoir in its place, counted once.
#[test]
fn test_a_first_frame_settled_into_gas_that_kept_its_reservoir_counts_it_once() {
    let mut evm = Run::new(ends(Vec::new())).evm();
    let mut result = FrameResult::new_call_oog(GAS_LIMIT, 0..0, RESERVOIR);
    let mut gas = GasTracker::new(GAS_LIMIT, 0, RESERVOIR);
    // A runner that settles a first frame it built itself opens the transaction's start first.
    revm_handler::lifecycle::open_transaction_start(&mut evm).expect("the start opens");

    Mainnet::default()
        .last_frame_result(&mut evm, &mut result, &mut gas)
        .expect("the frame settles");

    assert_eq!(gas.reservoir(), RESERVOIR);
    assert_eq!(evm.ctx.held, [(0, RESERVOIR)]);
}

/// A runner that builds the gas with `GasTracker::new`, with no reservoir, settles as before.
#[test]
fn test_a_first_frame_settled_into_gas_with_no_reservoir_settles() {
    let mut evm = Run::new(ends(Vec::new())).evm();
    let mut result = FrameResult::new_call_oog(GAS_LIMIT, 0..0, 0);
    let mut gas = GasTracker::new(GAS_LIMIT, 0, 0);
    // A runner that settles a first frame it built itself opens the transaction's start first.
    revm_handler::lifecycle::open_transaction_start(&mut evm).expect("the start opens");

    Mainnet::default()
        .last_frame_result(&mut evm, &mut result, &mut gas)
        .expect("the frame settles");

    assert_eq!(gas.reservoir(), 0);
}
