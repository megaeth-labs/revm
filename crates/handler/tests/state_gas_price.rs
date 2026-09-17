//! The EIP-8037 state gas pricing hook through the mainnet handler.
//!
//! The transactions run on a context that delegates everything to [`Context`] except
//! [`Host::state_gas_price`], which prices chosen sites away from the flat schedule and can be
//! told to fail. Each priced test fails if its call site reads the schedule directly; each
//! failure test checks that the cause the hook recorded is what the transaction returns.

use bytecode::opcode::{
    CALL, CREATE, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3, PUSH4, RETURN, REVERT, SSTORE, STOP,
};
use context::{
    result::{EVMError, ExecutionResult},
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Journal, LocalContext,
    TxEnv,
};
use context_interface::{
    cfg::{GasId, GasParams, StateGasCharge, StateGasSite},
    host::LoadError,
    journaled_state::AccountInfoLoad,
    result::ResultGas,
    transaction::{Authorization, RecoveredAuthority, RecoveredAuthorization},
    Database, Host, OutFrame,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    CreateInputs, CreateScheme, FrameInput, GasTracker, InstructionResult, SStoreResult,
    SelfDestructResult, SharedMemory, StateLoad,
};
use primitives::{
    address, constants::CALL_STACK_LIMIT, eip2780, eip8037, eip8038, hardfork::SpecId, Address,
    Bytes, HashMap, Log, StorageKey, StorageValue, TxKind, B256, U256,
};
use revm_handler::{
    execution, instructions::EthInstructions, EthFrame, EthPrecompiles, ExecuteEvm, FrameResult,
    ItemOrResult, MainBuilder, MainContext, MainnetContext, MainnetEvm,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;

/// Contract the transaction calls.
const CONTRACT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// An account with neither code nor balance.
const EMPTY: Address = address!("0x00000000000000000000000000000000000e3970");
/// An EOA a transaction calls without value.
const EOA: Address = address!("0x0000000000000000000000000000000000000eea");
/// An EIP-7702 authority that does not exist before the transaction.
const AUTHORITY: Address = address!("0x00000000000000000000000000000000000a0707");
/// What `AUTHORITY` delegates to.
const DELEGATE: Address = address!("0x00000000000000000000000000000000000de1e9");
/// The blake2f precompile, which fails on empty input and has no account.
const BLAKE2F: Address = address!("0x0000000000000000000000000000000000000009");

const GAS_LIMIT: u64 = 10_000_000;

/// The flat Amsterdam schedule.
const FLAT_SSTORE_SET: u64 = eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;
const FLAT_NEW_ACCOUNT: u64 = eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;
const FLAT_CREATE: u64 = eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;
const FLAT_CODE_BYTE: u64 = eip8037::CODE_DEPOSIT_PER_BYTE * eip8037::CPSB_GLAMSTERDAM;
const FLAT_DELEGATION_BYTES: u64 = eip8037::AUTH_BASE_BYTES * eip8037::CPSB_GLAMSTERDAM;

/// A price no flat schedule entry has.
const PRICE: u64 = 700_000;
/// A second price no flat schedule entry has, for a site that must not be confused with the first.
const OTHER_PRICE: u64 = 900_000;

/// What the failing hook records before it returns `None`.
const POISON_CAUSE: &str = "injected state gas price failure";

/// One state gas price lookup: which price, at which site.
type Lookup = (GasId, StateGasSite);

/// A lookup that fails once it has been answered `fail_after` times.
struct Poison {
    lookup: Lookup,
    fail_after: usize,
}

/// [`Context`] with a per-site state gas price table.
struct PricedContext {
    inner: MainnetContext<Db>,
    /// Unit prices that differ from the flat schedule.
    prices: HashMap<Lookup, u64>,
    poison: Option<Poison>,
    /// Every lookup, in order.
    lookups: Vec<Lookup>,
}

impl PricedContext {
    fn new(db: Db) -> Self {
        Self {
            inner: amsterdam(db),
            prices: HashMap::default(),
            poison: None,
            lookups: Vec::new(),
        }
    }

    fn with_price(mut self, lookup: Lookup, price: u64) -> Self {
        self.prices.insert(lookup, price);
        self
    }

    const fn with_poison(mut self, lookup: Lookup, fail_after: usize) -> Self {
        self.poison = Some(Poison { lookup, fail_after });
        self
    }
}

impl ContextTr for PricedContext {
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

impl ContextSetters for PricedContext {
    fn set_tx(&mut self, tx: Self::Tx) {
        self.inner.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.inner.set_block(block);
    }
}

impl Host for PricedContext {
    fn state_gas_price(&mut self, id: GasId, site: StateGasSite) -> Option<u64> {
        let lookup = (id, site);
        self.lookups.push(lookup);
        if let Some(poison) = &mut self.poison {
            if poison.lookup == lookup {
                if poison.fail_after == 0 {
                    *self.inner.error() = Err(ContextError::Custom(POISON_CAUSE.into()));
                    return None;
                }
                poison.fail_after -= 1;
            }
        }
        match self.prices.get(&lookup) {
            Some(price) => Some(*price),
            None => self.inner.state_gas_price(id, site),
        }
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

type Outcome = Result<ExecutionResult, EVMError<DbError>>;

/// An Amsterdam context with the transaction gas cap lifted, so the reservoir is empty and every
/// state charge spills onto the regular budget.
fn amsterdam(db: Db) -> MainnetContext<Db> {
    Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
        cfg.tx_gas_limit_cap = Some(u64::MAX);
    })
}

/// Runs `tx` on `ctx` and returns the outcome with the lookups the hook saw.
fn transact(ctx: PricedContext, tx: TxEnv) -> (Outcome, Vec<Lookup>) {
    let mut evm: MainnetEvm<PricedContext> = Evm::new(
        ctx,
        EthInstructions::new_mainnet_with_spec(SpecId::AMSTERDAM),
        EthPrecompiles::new(SpecId::AMSTERDAM),
    );
    let outcome = evm.transact(tx).map(|out| out.result);
    (outcome, evm.ctx.lookups)
}

/// Runs `tx` on `db` with the flat schedule and returns its gas.
fn flat_gas(db: Db, tx: TxEnv) -> ResultGas {
    let (outcome, _) = transact(PricedContext::new(db), tx);
    *outcome.expect("flat-priced transaction runs").gas()
}

/// A database where the sender is funded.
fn funded_db() -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    db
}

/// A funded database where [`CONTRACT`] holds `code` and `balance`.
fn db_with_contract(code: Vec<u8>, balance: u64) -> Db {
    let mut db = funded_db();
    let code = Bytecode::new_raw(Bytes::from(code));
    db.insert_account_info(
        CONTRACT,
        AccountInfo::new(U256::from(balance), 1, code.hash_slow(), code),
    );
    db
}

fn call_tx(to: Address, value: u64) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(TxKind::Call(to))
        .value(U256::from(value))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill()
}

fn create_tx(initcode: &[u8]) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(TxKind::Create)
        .data(Bytes::copy_from_slice(initcode))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill()
}

/// A valueless call to [`EOA`] carrying one authorization of the non-existent [`AUTHORITY`].
fn auth_tx() -> TxEnv {
    let mut tx = call_tx(EOA, 0);
    tx.set_recovered_authorization(vec![RecoveredAuthorization::new_unchecked(
        Authorization {
            chain_id: U256::ZERO,
            address: DELEGATE,
            nonce: 0,
        },
        RecoveredAuthority::Valid(AUTHORITY),
    )]);
    tx.derive_tx_type().unwrap();
    tx
}

/// `CALL(100_000, to, 1, 0, 0, 0, 0)`, leaving the success flag on the stack.
fn value_call(to: Address) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH1, 1, PUSH20];
    code.extend_from_slice(to.as_slice());
    code.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, CALL]);
    code
}

/// Contract code: `CALL(100_000, to, 1, 0, 0, 0, 0); STOP`.
fn value_call_code(to: Address) -> Vec<u8> {
    let mut code = value_call(to);
    code.push(STOP);
    code
}

/// Initcode that deploys nothing: `REVERT(0, 0)`.
const REVERTING_INITCODE: [u8; 3] = [PUSH0, PUSH0, REVERT];

/// Contract code: `CREATE(0, 29, 3)` of [`REVERTING_INITCODE`], then `STOP`.
fn create_reverting_code() -> Vec<u8> {
    let [a, b, c] = REVERTING_INITCODE;
    vec![
        // MSTORE(0, initcode): the initcode lands in memory bytes 29..32.
        PUSH3, a, b, c, PUSH0, MSTORE, //
        PUSH1, 3, PUSH1, 29, PUSH0, CREATE, STOP,
    ]
}

/// Initcode deploying 32 zero bytes: `RETURN(0, 32)`.
const INITCODE_32_BYTES: [u8; 4] = [PUSH1, 32, PUSH0, RETURN];

const fn new_account(address: Address) -> Lookup {
    (
        GasId::new_account_state_gas(),
        StateGasSite::account(address),
    )
}

const fn create_charge(address: Address) -> Lookup {
    (GasId::create_state_gas(), StateGasSite::account(address))
}

const fn delegation_bytes(address: Address) -> Lookup {
    (
        GasId::tx_eip7702_state_gas_bytecode(),
        StateGasSite::account(address),
    )
}

const fn code_deposit(address: Address) -> Lookup {
    (
        GasId::code_deposit_state_gas(),
        StateGasSite::account(address),
    )
}

fn priced(db: Db, lookup: Lookup, price: u64) -> PricedContext {
    PricedContext::new(db).with_price(lookup, price)
}

fn poisoned(db: Db, lookup: Lookup, fail_after: usize) -> PricedContext {
    PricedContext::new(db).with_poison(lookup, fail_after)
}

fn assert_fails_with_recorded_cause(outcome: Outcome) {
    match outcome {
        Err(EVMError::Custom(cause)) => assert_eq!(cause, POISON_CAUSE),
        other => panic!("expected the recorded cause, got {other:?}"),
    }
}

// EIP-2780 runtime phase.

#[test]
fn test_value_transfer_to_empty_recipient_is_priced_per_recipient() {
    let (outcome, lookups) = transact(
        priced(funded_db(), new_account(EMPTY), PRICE),
        call_tx(EMPTY, 1),
    );

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent_final(), PRICE);
    assert_eq!(
        result.gas().total_gas_spent(),
        eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS + eip2780::TX_VALUE_COST + PRICE
    );
    assert_eq!(lookups, [new_account(EMPTY)]);
}

#[test]
fn test_value_transfer_to_empty_recipient_lookup_failure_fails_tx() {
    let (outcome, lookups) = transact(
        poisoned(funded_db(), new_account(EMPTY), 0),
        call_tx(EMPTY, 1),
    );

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [new_account(EMPTY)]);
}

#[test]
fn test_failed_value_transfer_tx_refunds_the_recipient_price() {
    // blake2f rejects the empty input, so the first frame halts and the recipient leaf is never
    // created.
    let tx = call_tx(BLAKE2F, 1);
    let (outcome, lookups) = transact(priced(funded_db(), new_account(BLAKE2F), PRICE), tx);

    let result = outcome.unwrap();
    assert!(result.is_halt());
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(result.gas().total_gas_spent(), GAS_LIMIT);
    assert_eq!(lookups, [new_account(BLAKE2F), new_account(BLAKE2F)]);
}

#[test]
fn test_failed_value_transfer_tx_refund_lookup_failure_fails_tx() {
    let tx = call_tx(BLAKE2F, 1);
    let (outcome, lookups) = transact(poisoned(funded_db(), new_account(BLAKE2F), 1), tx);

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [new_account(BLAKE2F), new_account(BLAKE2F)]);
}

#[test]
fn test_creation_tx_target_is_priced_per_created_address() {
    let created = BENCH_CALLER.create(0);
    let (outcome, lookups) = transact(
        priced(funded_db(), create_charge(created), PRICE),
        create_tx(&[]),
    );

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent_final(), PRICE);
    assert_eq!(
        result.gas().total_gas_spent(),
        eip2780::TX_BASE_COST + eip8038::CREATE_ACCESS + PRICE
    );
    assert_eq!(lookups, [create_charge(created)]);
}

#[test]
fn test_creation_tx_first_frame_carries_the_priced_address() {
    // A create frame that fails before it runs refunds from the address on its inputs.
    let created = BENCH_CALLER.create(0);
    let mut ctx = priced(funded_db(), create_charge(created), PRICE);
    ctx.set_tx(create_tx(&[]));
    let mut gas = GasTracker::new(GAS_LIMIT, GAS_LIMIT, 0);

    let input = execution::create_init_frame(&mut ctx, &mut gas).unwrap();

    let Some(FrameInput::Create(inputs)) = input else {
        panic!("expected a create frame, got {input:?}");
    };
    assert!(inputs.charged_create_state_gas());
    assert_eq!(inputs.charged_state_gas_address(), created);
    assert_eq!(gas.state_gas_spent(), PRICE as i64);
}

#[test]
fn test_create_frame_failing_before_it_runs_refunds_the_priced_address() {
    // A create frame that cannot start (here: too deep) refunds from the address its inputs
    // carry. The CREATE opcode checks depth, balance and nonce before it charges, so the mainnet
    // flow never reaches this with a charge; the frame is driven directly.
    let created = CONTRACT.create(1);
    let mut ctx = priced(funded_db(), create_charge(created), PRICE);
    let mut inputs = CreateInputs::new(
        CONTRACT,
        CreateScheme::Create,
        U256::ZERO,
        Bytes::new(),
        GAS_LIMIT,
        0,
    );
    inputs.set_charged_create_state_gas(true);
    inputs.set_charged_state_gas_address(created);
    let mut frame = EthFrame::default();

    let result = EthFrame::make_create_frame::<_, ContextError<DbError>>(
        OutFrame::new_init(&mut frame),
        &mut ctx,
        CALL_STACK_LIMIT as usize + 1,
        SharedMemory::new(),
        Box::new(inputs),
    )
    .unwrap();

    let ItemOrResult::Result(result @ FrameResult::Create(_)) = result else {
        panic!("expected an early create result");
    };
    assert_eq!(result.instruction_result(), InstructionResult::CallTooDeep);
    let charge = result
        .refundable_state_gas_charge()
        .expect("the charge is refunded");
    let (id, site) = create_charge(created);
    assert_eq!(charge, StateGasCharge::one(id, site));
    assert_eq!(ctx.state_gas_charge(charge), Some(PRICE));
}

#[test]
fn test_creation_tx_target_lookup_failure_fails_tx() {
    let created = BENCH_CALLER.create(0);
    let (outcome, lookups) = transact(
        poisoned(funded_db(), create_charge(created), 0),
        create_tx(&[]),
    );

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [create_charge(created)]);
}

#[test]
fn test_reverted_creation_tx_refunds_the_created_address_price() {
    let created = BENCH_CALLER.create(0);
    let tx = create_tx(&REVERTING_INITCODE);
    let (outcome, lookups) = transact(
        priced(funded_db(), create_charge(created), PRICE),
        tx.clone(),
    );

    let result = outcome.unwrap();
    assert!(!result.is_success() && !result.is_halt(), "{result:?}");
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(*result.gas(), flat_gas(funded_db(), tx));
    assert_eq!(lookups, [create_charge(created), create_charge(created)]);
}

#[test]
fn test_reverted_creation_tx_refund_lookup_failure_fails_tx() {
    let created = BENCH_CALLER.create(0);
    let tx = create_tx(&REVERTING_INITCODE);
    let (outcome, lookups) = transact(poisoned(funded_db(), create_charge(created), 1), tx);

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [create_charge(created), create_charge(created)]);
}

#[test]
fn test_nonce_unchecked_reverted_creation_tx_refunds_the_charged_address() {
    // With the nonce check off, the sender's account nonce (1) differs from the transaction's
    // (0). The runtime phase charges the address the transaction nonce gives; the frame creates
    // the one the account nonce gives. The refund prices the address that was charged.
    let charged = BENCH_CALLER.create(0);
    let created = BENCH_CALLER.create(1);
    let ctx = || {
        let mut db = funded_db();
        db.insert_account_info(
            BENCH_CALLER,
            AccountInfo {
                nonce: 1,
                ..AccountInfo::from_balance(U256::from(10u128.pow(21)))
            },
        );
        let mut ctx = PricedContext::new(db);
        ctx.inner.cfg.disable_nonce_check = true;
        ctx
    };
    let tx = create_tx(&REVERTING_INITCODE);
    let ctx_priced = ctx()
        .with_price(create_charge(charged), PRICE)
        .with_price(create_charge(created), OTHER_PRICE);
    let (outcome, lookups) = transact(ctx_priced, tx.clone());
    let (flat, _) = transact(ctx(), tx);

    let result = outcome.unwrap();
    assert!(!result.is_success() && !result.is_halt(), "{result:?}");
    assert_eq!(lookups, [create_charge(charged), create_charge(charged)]);
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(*result.gas(), *flat.unwrap().gas());
}

// EIP-7702 authorizations under EIP-2780.

#[test]
fn test_eip7702_new_authority_leaf_is_priced_per_authority() {
    let (outcome, lookups) = transact(
        priced(funded_db(), new_account(AUTHORITY), PRICE),
        auth_tx(),
    );

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(
        result.gas().state_gas_spent_final(),
        PRICE + FLAT_DELEGATION_BYTES
    );
    assert_eq!(
        lookups,
        [new_account(AUTHORITY), delegation_bytes(AUTHORITY)]
    );
}

#[test]
fn test_eip7702_new_authority_leaf_lookup_failure_fails_tx() {
    let (outcome, lookups) = transact(poisoned(funded_db(), new_account(AUTHORITY), 0), auth_tx());

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [new_account(AUTHORITY)]);
}

#[test]
fn test_eip7702_delegation_bytes_are_priced_per_authority() {
    let (outcome, lookups) = transact(
        priced(funded_db(), delegation_bytes(AUTHORITY), PRICE),
        auth_tx(),
    );

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(
        result.gas().state_gas_spent_final(),
        FLAT_NEW_ACCOUNT + PRICE
    );
    assert_eq!(
        lookups,
        [new_account(AUTHORITY), delegation_bytes(AUTHORITY)]
    );
}

#[test]
fn test_eip7702_delegation_bytes_lookup_failure_fails_tx() {
    let (outcome, lookups) = transact(
        poisoned(funded_db(), delegation_bytes(AUTHORITY), 0),
        auth_tx(),
    );

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(
        lookups,
        [new_account(AUTHORITY), delegation_bytes(AUTHORITY)]
    );
}

// Code deposit.

#[test]
fn test_code_deposit_is_priced_per_byte_at_the_created_address() {
    let created = BENCH_CALLER.create(0);
    let per_byte = 2_000;
    let tx = create_tx(&INITCODE_32_BYTES);
    let (outcome, lookups) = transact(priced(funded_db(), code_deposit(created), per_byte), tx);

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(
        result.gas().state_gas_spent_final(),
        FLAT_CREATE + 32 * per_byte
    );
    assert_eq!(lookups, [create_charge(created), code_deposit(created)]);
}

#[test]
fn test_code_deposit_lookup_failure_fails_tx() {
    let created = BENCH_CALLER.create(0);
    let tx = create_tx(&INITCODE_32_BYTES);
    let (outcome, lookups) = transact(poisoned(funded_db(), code_deposit(created), 0), tx);

    assert_fails_with_recorded_cause(outcome);
    // The failed deposit ends the first frame; what the handler does with that frame's own
    // charges before it returns the error is not what this test is about.
    assert!(
        lookups.starts_with(&[create_charge(created), code_deposit(created)]),
        "{lookups:?}"
    );
}

// Refunds of the CALL and CREATE opcode charges.

#[test]
fn test_failed_value_call_refunds_the_callee_price() {
    // CONTRACT has no balance, so the transfer fails when the child frame is created.
    let db = || db_with_contract(value_call_code(EMPTY), 0);
    let tx = call_tx(CONTRACT, 0);
    let (outcome, lookups) = transact(priced(db(), new_account(EMPTY), PRICE), tx.clone());

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(*result.gas(), flat_gas(db(), tx));
    assert_eq!(lookups, [new_account(EMPTY), new_account(EMPTY)]);
}

#[test]
fn test_failed_value_call_refund_lookup_failure_fails_tx() {
    let db = db_with_contract(value_call_code(EMPTY), 0);
    let (outcome, lookups) = transact(poisoned(db, new_account(EMPTY), 1), call_tx(CONTRACT, 0));

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [new_account(EMPTY), new_account(EMPTY)]);
}

#[test]
fn test_reverted_create_refunds_the_created_address_price() {
    // CONTRACT is at nonce 1.
    let created = CONTRACT.create(1);
    let db = || db_with_contract(create_reverting_code(), 0);
    let tx = call_tx(CONTRACT, 0);
    let (outcome, lookups) = transact(priced(db(), create_charge(created), PRICE), tx.clone());

    let result = outcome.unwrap();
    assert!(result.is_success());
    assert_eq!(result.gas().state_gas_spent_final(), 0);
    assert_eq!(*result.gas(), flat_gas(db(), tx));
    assert_eq!(lookups, [create_charge(created), create_charge(created)]);
}

#[test]
fn test_reverted_create_refund_lookup_failure_fails_tx() {
    let created = CONTRACT.create(1);
    let db = db_with_contract(create_reverting_code(), 0);
    let (outcome, lookups) = transact(
        poisoned(db, create_charge(created), 1),
        call_tx(CONTRACT, 0),
    );

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [create_charge(created), create_charge(created)]);
}

// Opcode charges.

#[test]
fn test_sstore_lookup_failure_fails_tx() {
    // SSTORE(1, 1); STOP
    let db = db_with_contract(vec![PUSH1, 1, PUSH1, 1, SSTORE, STOP], 0);
    let slot = (
        GasId::sstore_set_state_gas(),
        StateGasSite::slot(CONTRACT, StorageKey::from(1)),
    );
    let (outcome, lookups) = transact(poisoned(db, slot, 0), call_tx(CONTRACT, 0));

    assert_fails_with_recorded_cause(outcome);
    assert_eq!(lookups, [slot]);
}

// The default.

#[test]
fn test_default_host_charges_the_flat_schedule() {
    // SSTORE(1, 1); POP(CALL(100_000, EMPTY, 1, 0, 0, 0, 0));
    // POP(CREATE(0, 28, 4)) of the initcode `RETURN(0, 2)`; STOP
    let mut code = vec![PUSH1, 1, PUSH1, 1, SSTORE];
    code.extend(value_call(EMPTY));
    code.push(POP);
    code.extend([PUSH4, PUSH1, 2, PUSH0, RETURN, PUSH0, MSTORE]);
    code.extend([PUSH1, 4, PUSH1, 28, PUSH0, CREATE, POP]);
    code.push(STOP);
    let db = || db_with_contract(code.clone(), 10);
    let tx = call_tx(CONTRACT, 0);

    let mut evm = amsterdam(db()).build_mainnet();
    let result = evm.transact(tx.clone()).unwrap().result;

    assert!(result.is_success());
    assert_eq!(
        result.gas().state_gas_spent_final(),
        FLAT_SSTORE_SET + FLAT_NEW_ACCOUNT + FLAT_CREATE + 2 * FLAT_CODE_BYTE
    );
    // What the same transaction costs on the tree before the hook existed.
    assert_eq!(result.gas().total_gas_spent(), 519_342);
    // An override that prices nothing differently is the default.
    assert_eq!(*result.gas(), flat_gas(db(), tx));
}
