//! An outcome an inspector builds to answer a start settles its reservoir as built, in place of
//! the caller's, and a consumer that keeps part of the caller's reservoir adds back what the
//! caller held.
//!
//! The caller's start copies its reservoir into the input, and the answer's reservoir replaces the
//! caller's, as a frame's does. An answer built on `Gas::new` carries no reservoir: by default the
//! caller is left with none, as upstream's settlement leaves it; with 98% forwarded by a consumer
//! that keeps the rest for the caller ([`KeepingContext`]), the caller is left with the 2% it kept.
//! An answer that carries the input's reservoir gives the caller back what it forwarded.
//!
//! A creation's answer settles through the same arm as a call's: the caller also paid the
//! creation's upfront state charge before the forward, which an answer that copies the input's
//! charge gives back and one built with `CreateOutcome::new` does not.

use context::{
    context_interface::{
        cfg::{GasId, GasParams, StateGasSite},
        context::{SStoreResult, SelfDestructResult, StateLoad},
        host::{FrameSettle, FrameSplit, FrameStart, LifecycleEnd, LoadError},
        journaled_state::AccountInfoLoad,
        Database,
    },
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Host, Journal,
    LocalContext, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, MainContext, MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome, Gas,
    GasTracker, InstructionResult, InterpreterResult,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, TxKind, B256, U256,
};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{CALL, CREATE, POP, PUSH0, PUSH1, PUSH20, PUSH3, SSTORE, STOP},
    AccountInfo, Bytecode,
};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type Ctx = KeepingContext;

const SPEC: SpecId = SpecId::AMSTERDAM;
const GAS_LIMIT: u64 = 3_000_000;
/// The reservoir the transaction starts with: the regular cap is this far below [`GAS_LIMIT`].
const RESERVOIR: u64 = 1_000_000;
/// State gas of a new storage slot on Amsterdam's schedule, which [`PARENT`] pays.
const SLOT: u64 = 97_920;
/// What [`PARENT`] holds when it calls [`OK`].
const BEFORE_THE_CALL: u64 = RESERVOIR - SLOT;
/// Its share at 98: all but the 2% the consumer keeps, rounded down (18,041.6 → 18,041).
const SHARE: u64 = 884_039;

/// The state charge a creation's start pays for its account on Amsterdam's schedule, before the
/// forward.
const ACCOUNT: u64 = 183_600;
/// What [`PARENT`] holds when it creates, once the creation's charge is paid.
const BEFORE_THE_CREATION: u64 = BEFORE_THE_CALL - ACCOUNT;
/// Its share at 98: all but the 2% the consumer keeps, rounded down (14,369.6 → 14,369).
const CREATION_SHARE: u64 = 704_111;

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

/// Writes a new slot, calls [`OK`] (or creates a contract with empty init code) and stops.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Stops, when it runs.
const OK: Address = address!("0x0000000000000000000000000000000000000c00");

/// What [`PARENT`] starts after writing its slot.
#[derive(Clone, Copy, Debug)]
enum Start {
    /// A call to [`OK`].
    Call,
    /// A creation with empty init code.
    Create,
}

/// What the inspector does with the call to [`OK`].
#[derive(Clone, Copy, Debug)]
enum Answer {
    /// Lets it run.
    Runs,
    /// Answers it with a revert built on `Gas::new`, as a tool does.
    AsBuilt,
    /// Answers it with a revert that carries the input's reservoir.
    WithTheShares,
}

/// Answers the call to [`OK`], or the creation, as told, and records the reservoir the start was
/// given and [`PARENT`]'s reservoir as its frame returned, before the transaction settled it.
struct Answering {
    answer: Answer,
    given: Option<u64>,
    parent_returned: Option<u64>,
}

impl Answering {
    /// The gas of the answer to a start of `gas_limit` given `reservoir`, or `None` to let the
    /// start run.
    const fn gas(&mut self, gas_limit: u64, reservoir: u64) -> Option<Gas> {
        self.given = Some(reservoir);
        match self.answer {
            Answer::Runs => None,
            Answer::AsBuilt => Some(Gas::new(gas_limit)),
            Answer::WithTheShares => Some(Gas::new_with_regular_gas_and_reservoir(
                gas_limit, reservoir,
            )),
        }
    }
}

impl Inspector<Ctx, EthInterpreter> for Answering {
    fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        if inputs.target_address != OK {
            return None;
        }
        let gas = self.gas(inputs.gas_limit, inputs.reservoir)?;
        Some(CallOutcome::new(
            InterpreterResult::new(InstructionResult::Revert, Bytes::new(), gas),
            inputs.return_memory_offset.clone(),
        ))
    }

    fn create(&mut self, _: &mut Ctx, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        let gas = self.gas(inputs.gas_limit(), inputs.reservoir())?;
        let mut outcome = CreateOutcome::new(
            InterpreterResult::new(InstructionResult::Revert, Bytes::new(), gas),
            None,
        );
        if let Answer::WithTheShares = self.answer {
            outcome.charged_create_state_gas = inputs.charged_create_state_gas();
            outcome.charged_state_gas_address = inputs.charged_state_gas_address();
        }
        Some(outcome)
    }

    fn call_end(&mut self, _: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
        if inputs.target_address == PARENT {
            self.parent_returned = Some(outcome.result.gas.reservoir());
        }
    }
}

/// [`PARENT`]'s reservoir as its frame returned, and the reservoir its `start` was given, with
/// `percent` per cent of a caller's reservoir forwarded by a consumer that keeps the rest
/// ([`KeepingContext`]), or the whole reservoir.
fn run(answer: Answer, percent: Option<u64>, start: Start) -> (u64, u64) {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    let mut parent = vec![PUSH1, 1, PUSH0, SSTORE];
    match start {
        // `CALL(100_000, OK, 0, 0, 0, 0, 0)`.
        Start::Call => {
            parent.extend_from_slice(&[PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20]);
            parent.extend_from_slice(OK.as_slice());
            parent.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, CALL]);
        }
        // `CREATE(0, 0, 0)`.
        Start::Create => parent.extend_from_slice(&[PUSH0, PUSH0, PUSH0, CREATE]),
    }
    parent.extend_from_slice(&[POP, STOP]);
    for (address, code) in [(PARENT, parent), (OK, vec![STOP])] {
        db.insert_account_info(
            address,
            AccountInfo::default()
                .with_nonce(1)
                .with_code(Bytecode::new_raw(code.into())),
        );
    }
    let ctx = Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(SPEC);
        cfg.tx_gas_limit_cap = Some(GAS_LIMIT - RESERVOIR);
    });
    let ctx = KeepingContext {
        inner: ctx,
        percent,
        held: Vec::new(),
    };
    let mut evm: Evm<
        Ctx,
        Answering,
        EthInstructions<EthInterpreter, Ctx>,
        EthPrecompiles,
        EthFrame,
    > = Evm::new_with_inspector(
        ctx,
        Answering {
            answer,
            given: None,
            parent_returned: None,
        },
        EthInstructions::new_mainnet_with_spec(SPEC),
        EthPrecompiles::new(SPEC),
    );
    let tx = TxEnv::builder_for_bench()
        .kind(TxKind::Call(PARENT))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill();
    let outcome = evm.inspect_tx(tx).expect("the transaction runs");
    assert!(outcome.result.is_success(), "{:?}", outcome.result);
    (
        evm.inspector.parent_returned.expect("PARENT returned"),
        evm.inspector.given.expect("the call to OK started"),
    )
}

#[test]
fn test_an_answer_the_inspector_builds_settles_its_reservoir_as_built() {
    for (answer, whole, kept_98) in [
        (Answer::Runs, BEFORE_THE_CALL, BEFORE_THE_CALL),
        // The settlement gives the caller the answer's empty reservoir in place of its own; the
        // consumer that kept 2% adds that back.
        (Answer::AsBuilt, 0, BEFORE_THE_CALL - SHARE),
        (Answer::WithTheShares, BEFORE_THE_CALL, BEFORE_THE_CALL),
    ] {
        assert_eq!(
            run(answer, None, Start::Call),
            (whole, BEFORE_THE_CALL),
            "{answer:?}, the whole reservoir forwarded"
        );
        assert_eq!(
            run(answer, Some(98), Start::Call),
            (kept_98, SHARE),
            "{answer:?}, 98% forwarded"
        );
    }
}

/// A creation's answer settles as a call's: as built, the caller gets back the part the consumer
/// kept, and keeps the creation's charge; with the shares and the input's charges, the caller gets back what it held
/// before the creation's start.
#[test]
fn test_an_answer_the_inspector_builds_for_a_creation_settles_its_reservoir_as_built() {
    for (answer, whole, kept_98) in [
        (Answer::Runs, BEFORE_THE_CREATION, BEFORE_THE_CREATION),
        (Answer::AsBuilt, 0, BEFORE_THE_CREATION - CREATION_SHARE),
        (Answer::WithTheShares, BEFORE_THE_CALL, BEFORE_THE_CALL),
    ] {
        assert_eq!(
            run(answer, None, Start::Create),
            (whole, BEFORE_THE_CREATION),
            "{answer:?}, the whole reservoir forwarded"
        );
        assert_eq!(
            run(answer, Some(98), Start::Create),
            (kept_98, CREATION_SHARE),
            "{answer:?}, 98% forwarded"
        );
    }
}
