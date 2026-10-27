//! The code-deposit admission hook ([`ContextTr::admit_code_deposit`]) through the mainnet
//! handler.
//!
//! The transactions run on a context that delegates everything to [`Context`] except the
//! admission hook, which records every deposit it is offered and admits, refuses with a named
//! output, or fails fatally. Each case is compared with a twin: an admitting context against
//! [`Context`] itself, and a refused deposit against the same creation whose init code reverts.
//! One test calls `return_create` directly, on a frame whose regular gas is partly withheld.

use bytecode::opcode::{
    CREATE, MSTORE, MSTORE8, PUSH0, PUSH1, PUSH3, PUSH4, RETURN, RETURNDATASIZE, REVERT, SSTORE,
    STOP,
};
use context::{
    result::{EVMError, ExecutionResult, ResultAndState},
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Journal, LocalContext,
    TxEnv,
};
use context_interface::{
    cfg::{gas::GasTracker, GasId, GasParams, StateGasSite},
    context::CodeDeposit,
    host::LoadError,
    journaled_state::AccountInfoLoad,
    Cfg, Database, Host, JournalTr,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    Gas, InstructionResult, InterpreterResult, SStoreResult, SelfDestructResult, StateLoad,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, TxKind, B256,
    KECCAK_EMPTY, U256,
};
use revm_handler::{
    instructions::EthInstructions, return_create, EthPrecompiles, ExecuteEvm, MainBuilder,
    MainContext, MainnetContext, MainnetEvm,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type Outcome = Result<ResultAndState, EVMError<DbError>>;

/// Contract the call transactions call.
const CONTRACT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Nonce of [`CONTRACT`], so the address its `CREATE` deploys at is known.
const CONTRACT_NONCE: u64 = 1;
/// Gas limit of every transaction.
const GAS_LIMIT: u64 = 1_000_000;
/// The output a refusing context names.
const REFUSAL: &[u8] = b"refused";
/// The cause a fatally refusing context records.
const FATAL_CAUSE: &str = "admission lookup failed";

/// Init code that deploys 32 zero bytes.
const RETURN_32: [u8; 4] = [PUSH1, 32, PUSH0, RETURN];
/// Init code that reverts with 32 zero bytes, at the gas [`RETURN_32`] runs on.
const REVERT_32: [u8; 4] = [PUSH1, 32, PUSH0, REVERT];
/// Init code that deploys one `0xEF` byte, which EIP-3541 refuses.
const RETURN_EF: [u8; 8] = [PUSH1, 0xEF, PUSH0, MSTORE8, PUSH1, 1, PUSH0, RETURN];

/// What the context does with a deposit it is offered.
#[derive(Clone, Copy)]
enum Policy {
    Admit,
    Refuse,
    RefuseFatally,
}

/// A deposit the context was offered, with the charges it read off it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Offered {
    address: Address,
    code: Bytes,
    regular_gas: u64,
    state_gas: u64,
    history_gas: u64,
}

/// [`Context`] with an admission hook that follows a [`Policy`].
struct AdmissionContext {
    inner: MainnetContext<Db>,
    policy: Policy,
    offered: Vec<Offered>,
    /// The creating frame's gas before and after the deposit's charges, of every deposit offered.
    offered_gas: Vec<(GasTracker, GasTracker)>,
    /// The site of every code-deposit state gas price lookup, in order.
    deposit_prices: Vec<StateGasSite>,
}

impl ContextTr for AdmissionContext {
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

    fn admit_code_deposit(&mut self, deposit: &CodeDeposit<'_>) -> Result<(), Bytes> {
        self.offered.push(Offered {
            address: deposit.address,
            code: deposit.code.clone(),
            regular_gas: deposit.regular_gas(),
            state_gas: deposit.state_gas(),
            history_gas: deposit.history_gas(),
        });
        self.offered_gas
            .push((*deposit.gas_before, *deposit.gas_after));
        match self.policy {
            Policy::Admit => Ok(()),
            Policy::Refuse => Err(Bytes::from_static(REFUSAL)),
            Policy::RefuseFatally => {
                *self.inner.error() = Err(ContextError::Custom(FATAL_CAUSE.into()));
                Err(Bytes::new())
            }
        }
    }
}

impl ContextSetters for AdmissionContext {
    fn set_tx(&mut self, tx: Self::Tx) {
        self.inner.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.inner.set_block(block);
    }
}

impl Host for AdmissionContext {
    fn state_gas_price(&mut self, id: GasId, site: StateGasSite) -> Option<u64> {
        if id == GasId::code_deposit_state_gas() {
            self.deposit_prices.push(site);
        }
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

/// A spec, and what its schedule charges per byte of deposited code as history gas.
#[derive(Clone, Copy, Debug)]
struct Schedule {
    spec: SpecId,
    history_per_byte: u64,
}

/// Before EIP-8037, where the deposit is one regular charge; under it, where the code's hash and
/// state gas are charged too; and under it with history bytes priced, where the history charge
/// comes last.
const SCHEDULES: [Schedule; 3] = [
    Schedule {
        spec: SpecId::OSAKA,
        history_per_byte: 0,
    },
    Schedule {
        spec: SpecId::AMSTERDAM,
        history_per_byte: 0,
    },
    Schedule {
        spec: SpecId::AMSTERDAM,
        history_per_byte: 10,
    },
];

/// A context under `schedule` on `db`.
fn context(db: Db, schedule: Schedule) -> MainnetContext<Db> {
    Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(schedule.spec);
        cfg.gas_params
            .override_gas([(GasId::code_deposit_history_gas(), schedule.history_per_byte)]);
    })
}

/// An admission context under `schedule` on `db`, following `policy`.
fn admission_context(db: Db, schedule: Schedule, policy: Policy) -> AdmissionContext {
    AdmissionContext {
        inner: context(db, schedule),
        policy,
        offered: Vec::new(),
        offered_gas: Vec::new(),
        deposit_prices: Vec::new(),
    }
}

/// Runs `tx` on an admission context following `policy`, and returns the outcome with the
/// context, which holds what it was offered and asked.
fn run(db: Db, schedule: Schedule, policy: Policy, tx: TxEnv) -> (Outcome, AdmissionContext) {
    let ctx = admission_context(db, schedule, policy);
    let mut evm: MainnetEvm<AdmissionContext> = Evm::new(
        ctx,
        EthInstructions::new_mainnet_with_spec(schedule.spec),
        EthPrecompiles::new(schedule.spec),
    );
    let outcome = evm.transact(tx);
    (outcome, evm.ctx)
}

/// Runs `tx` on an admission context following `policy`, and returns the outcome with the
/// deposits it was offered.
fn transact(db: Db, schedule: Schedule, policy: Policy, tx: TxEnv) -> (Outcome, Vec<Offered>) {
    let (outcome, ctx) = run(db, schedule, policy, tx);
    (outcome, ctx.offered)
}

/// Runs `tx` on [`Context`] itself, which has the default hook.
fn transact_plain(db: Db, schedule: Schedule, tx: TxEnv) -> Outcome {
    context(db, schedule).build_mainnet().transact(tx)
}

/// A database with a funded sender, and [`CONTRACT`] holding `code` when given.
fn db(code: Option<Vec<u8>>) -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    if let Some(code) = code {
        db.insert_account_info(
            CONTRACT,
            AccountInfo::default()
                .with_nonce(CONTRACT_NONCE)
                .with_code(Bytecode::new_raw(code.into())),
        );
    }
    db
}

fn create_tx(initcode: &[u8], gas_limit: u64) -> TxEnv {
    TxEnv::builder()
        .caller(BENCH_CALLER)
        .kind(TxKind::Create)
        .data(Bytes::copy_from_slice(initcode))
        .gas_limit(gas_limit)
        .build()
        .unwrap()
}

fn call_tx() -> TxEnv {
    TxEnv::builder()
        .caller(BENCH_CALLER)
        .kind(TxKind::Call(CONTRACT))
        .gas_limit(GAS_LIMIT)
        .build()
        .unwrap()
}

/// Code that runs a four-byte `initcode` through `CREATE`, then stores the address it pushed in
/// slot 0 and the size of the return data it left in slot 1.
fn create_and_record(initcode: [u8; 4]) -> Vec<u8> {
    let mut code = vec![PUSH4];
    code.extend_from_slice(&initcode);
    code.extend_from_slice(&[PUSH0, MSTORE]);
    // CREATE(value 0, offset 28, size 4): the init code sits in the last four bytes of word 0.
    code.extend_from_slice(&[PUSH1, 4, PUSH1, 28, PUSH0, CREATE]);
    code.extend_from_slice(&[PUSH0, SSTORE]);
    code.extend_from_slice(&[RETURNDATASIZE, PUSH1, 1, SSTORE, STOP]);
    code
}

/// The address a top-level creation from the funded sender deploys at.
fn created_by_sender() -> Address {
    BENCH_CALLER.create(0)
}

fn slot(outcome: &ResultAndState, address: Address, key: u64) -> U256 {
    outcome.state[&address]
        .storage
        .get(&StorageKey::from(key))
        .map(|slot| slot.present_value)
        .unwrap_or_default()
}

fn code_hash(outcome: &ResultAndState, address: Address) -> B256 {
    outcome
        .state
        .get(&address)
        .map_or(KECCAK_EMPTY, |account| account.info.code_hash)
}

/// What a deposit of `len` bytes charges under `schedule`, with the flat state gas price: the
/// regular gas, the state gas and the history gas.
fn deposit_charges(schedule: Schedule, len: usize) -> (u64, u64, u64) {
    let params: GasParams = context(Db::default(), schedule).cfg.gas_params;
    let eip8037 = schedule.spec.is_enabled_in(SpecId::AMSTERDAM);
    let regular = params.code_deposit_cost(len)
        + if eip8037 {
            params.keccak256_cost(len)
        } else {
            0
        };
    let state = if eip8037 {
        params.code_deposit_state_gas(len)
    } else {
        0
    };
    let history = if eip8037 {
        params.code_deposit_history_gas(len)
    } else {
        0
    };
    (regular, state, history)
}

/// The hook is offered the deposit once, with its address, its code and what it charged, and an
/// admitting hook leaves the transaction exactly as [`Context`] runs it.
#[test]
fn test_the_hook_is_offered_the_code_and_its_charges() {
    for spec in SCHEDULES {
        let tx = create_tx(&RETURN_32, GAS_LIMIT);
        let (outcome, offered) = transact(db(None), spec, Policy::Admit, tx.clone());
        let (regular_gas, state_gas, history_gas) = deposit_charges(spec, 32);

        assert_eq!(
            offered,
            vec![Offered {
                address: created_by_sender(),
                code: Bytes::from_static(&[0; 32]),
                regular_gas,
                state_gas,
                history_gas,
            }],
            "{spec:?}"
        );
        assert!(regular_gas > 0, "{spec:?}: the deposit charges regular gas");
        assert_eq!(state_gas > 0, spec.spec == SpecId::AMSTERDAM, "{spec:?}");
        assert_eq!(history_gas, 32 * spec.history_per_byte, "{spec:?}");

        let outcome = outcome.expect("the creation runs");
        assert!(
            outcome.result.is_success(),
            "{spec:?}: {:?}",
            outcome.result
        );
        assert_eq!(
            outcome,
            transact_plain(db(None), spec, tx).unwrap(),
            "{spec:?}: an admitting hook changes nothing"
        );
        assert_ne!(code_hash(&outcome, created_by_sender()), KECCAK_EMPTY);
    }
}

/// A refused top-level creation reverts with the named output, deploys nothing and pays none of
/// the deposit: it ends as the same creation whose init code reverts at the point it returned.
#[test]
fn test_a_refused_creation_reverts_with_the_named_output() {
    for spec in SCHEDULES {
        let (refused, offered) = transact(
            db(None),
            spec,
            Policy::Refuse,
            create_tx(&RETURN_32, GAS_LIMIT),
        );
        let reverted = transact_plain(db(None), spec, create_tx(&REVERT_32, GAS_LIMIT)).unwrap();
        let refused = refused.expect("a refusal is not an error");

        assert_eq!(offered.len(), 1, "{spec:?}");
        match &refused.result {
            ExecutionResult::Revert { output, .. } => assert_eq!(output.as_ref(), REFUSAL),
            other => panic!("{spec:?}: expected a revert, got {other:?}"),
        }
        assert!(matches!(reverted.result, ExecutionResult::Revert { .. }));
        assert_eq!(
            refused.result.gas(),
            reverted.result.gas(),
            "{spec:?}: the refusal pays what the reverting twin pays"
        );
        assert_eq!(refused.state, reverted.state, "{spec:?}");
        assert_eq!(code_hash(&refused, created_by_sender()), KECCAK_EMPTY);
    }
}

/// A refused nested creation is a failed `CREATE` its caller survives: the caller gets a zero
/// address and the named output as return data, and pays what it pays when the init code reverts.
#[test]
fn test_a_refused_nested_creation_fails_the_create_and_its_caller_survives() {
    let created = CONTRACT.create(CONTRACT_NONCE);
    for spec in SCHEDULES {
        let (refused, offered) = transact(
            db(Some(create_and_record(RETURN_32))),
            spec,
            Policy::Refuse,
            call_tx(),
        );
        let (admitted, _) = transact(
            db(Some(create_and_record(RETURN_32))),
            spec,
            Policy::Admit,
            call_tx(),
        );
        let reverted = transact_plain(db(Some(create_and_record(REVERT_32))), spec, call_tx())
            .expect("the twin runs");
        let (refused, admitted) = (refused.unwrap(), admitted.unwrap());

        assert_eq!(offered.len(), 1, "{spec:?}");
        assert_eq!(offered[0].address, created, "{spec:?}");
        assert!(
            refused.result.is_success(),
            "{spec:?}: {:?}",
            refused.result
        );

        assert_eq!(
            slot(&refused, CONTRACT, 0),
            U256::ZERO,
            "{spec:?}: CREATE pushed 0"
        );
        assert_eq!(
            slot(&refused, CONTRACT, 1),
            U256::from(REFUSAL.len()),
            "{spec:?}: the named output is the return data"
        );
        assert_eq!(code_hash(&refused, created), KECCAK_EMPTY, "{spec:?}");

        assert_eq!(
            slot(&admitted, CONTRACT, 0),
            U256::from_be_slice(created.as_slice()),
            "{spec:?}"
        );
        assert_eq!(slot(&reverted, CONTRACT, 1), U256::from(32), "{spec:?}");
        assert_eq!(
            refused.result.gas(),
            reverted.result.gas(),
            "{spec:?}: the refusal pays what the reverting twin pays"
        );
    }
}

/// The deposit's state gas is priced before the deposit is offered, so the price lookup is made
/// whether or not it is admitted: once, at the deployed address, for an admitted and a refused
/// deposit alike, and not at all where the schedule does not price deposited code.
#[test]
fn test_the_deposit_is_priced_whether_or_not_it_is_admitted() {
    for spec in SCHEDULES {
        let expected = if spec.spec == SpecId::AMSTERDAM {
            vec![StateGasSite::account(created_by_sender())]
        } else {
            Vec::new()
        };
        for (name, policy) in [("admitted", Policy::Admit), ("refused", Policy::Refuse)] {
            let (outcome, ctx) = run(db(None), spec, policy, create_tx(&RETURN_32, GAS_LIMIT));

            assert!(outcome.is_ok(), "{spec:?}, {name}");
            assert_eq!(ctx.offered.len(), 1, "{spec:?}, {name}");
            assert_eq!(ctx.deposit_prices, expected, "{spec:?}, {name}");
        }
    }
}

/// A creation `return_create` fails itself is never offered, and fails exactly as it does with
/// the default hook: init code that reverts, code starting with `0xEF`, code over the size limit,
/// and a frame that cannot pay the deposit.
#[test]
fn test_creations_return_create_fails_itself_are_never_offered() {
    for spec in SCHEDULES {
        let admitted_gas = transact_plain(db(None), spec, create_tx(&RETURN_32, GAS_LIMIT))
            .unwrap()
            .result
            .gas()
            .total_gas_spent();
        let max_code_size = context(db(None), spec).cfg().max_code_size();
        let size = (max_code_size + 1).to_be_bytes();
        let oversized = [PUSH3, size[5], size[6], size[7], PUSH0, RETURN];

        for (name, tx) in [
            ("reverting init code", create_tx(&REVERT_32, GAS_LIMIT)),
            ("0xEF code", create_tx(&RETURN_EF, GAS_LIMIT)),
            ("oversized code", create_tx(&oversized, GAS_LIMIT)),
            ("unpayable deposit", create_tx(&RETURN_32, admitted_gas - 1)),
        ] {
            let (outcome, offered) = transact(db(None), spec, Policy::Refuse, tx.clone());
            let outcome = outcome.expect("the creation runs");

            assert!(offered.is_empty(), "{spec:?}, {name}: offered {offered:?}");
            assert!(!outcome.result.is_success(), "{spec:?}, {name}");
            assert_eq!(
                outcome,
                transact_plain(db(None), spec, tx).unwrap(),
                "{spec:?}, {name}"
            );
        }
    }
}

/// Calls `return_create` for a creation at [`CONTRACT`] that returned `code` with `gas`, under
/// `spec`, on an admission context following `policy`, and returns the creation's result with the
/// context.
fn return_create_on(
    spec: SpecId,
    policy: Policy,
    gas: Gas,
    code: Bytes,
) -> (InterpreterResult, AdmissionContext) {
    let schedule = Schedule {
        spec,
        history_per_byte: 0,
    };
    let mut ctx = admission_context(db(None), schedule, policy);
    ctx.journal_mut().load_account(CONTRACT).unwrap();
    let checkpoint = ctx.journal_mut().checkpoint();
    let mut result = InterpreterResult::new(InstructionResult::Return, code, gas);
    return_create(&mut ctx, checkpoint, &mut result, CONTRACT);
    (result, ctx)
}

/// Before Homestead a frame that cannot pay the code deposit does not fail: `return_create`
/// deploys empty code, and offers that deposit with empty code and no charge. When the failed
/// charge crossed the frame's withheld gas, its record is on `gas_after` and not on `gas_before`:
/// an admitted deposit returns with it, and a refused one, put back to `gas_before`, discards it.
/// From Homestead on the same frame runs out of gas and is not offered.
#[test]
fn test_a_pre_homestead_deposit_the_frame_cannot_pay_is_offered_with_its_crossing() {
    // 100 spendable and 200 withheld; one byte of code costs 200 to deposit, so the charge
    // crosses.
    let mut gas = Gas::new(300);
    gas.withhold(200);
    let code = Bytes::from_static(&[0]);
    let parts = |gas: &GasTracker| (gas.spendable(), gas.withheld());

    for (name, policy) in [("admitted", Policy::Admit), ("refused", Policy::Refuse)] {
        let (result, ctx) = return_create_on(SpecId::FRONTIER, policy, gas, code.clone());

        assert_eq!(
            ctx.offered,
            vec![Offered {
                address: CONTRACT,
                code: Bytes::new(),
                regular_gas: 0,
                state_gas: 0,
                history_gas: 0,
            }],
            "{name}"
        );
        let [(before, after)] = ctx.offered_gas[..] else {
            panic!("{name}: offered {} times", ctx.offered_gas.len());
        };
        assert_eq!(parts(&before), (100, 200), "{name}");
        assert_eq!(parts(&after), (100, 200), "{name}: nothing was charged");
        assert!(before.withheld_crossing().is_none(), "{name}");
        assert!(
            after.withheld_crossing().is_some(),
            "{name}: the crossing is on gas_after"
        );

        match policy {
            Policy::Admit => {
                assert_eq!(result.result, InstructionResult::Return);
                assert!(result.output.is_empty());
                assert!(
                    result.gas.withheld_crossing().is_some(),
                    "the record stays on a creation that returns"
                );
            }
            _ => {
                assert_eq!(result.result, InstructionResult::Revert);
                assert_eq!(result.output.as_ref(), REFUSAL);
                assert_eq!(result.gas, gas, "the gas is put back to gas_before");
                assert!(
                    result.gas.withheld_crossing().is_none(),
                    "the refusal discards the record"
                );
            }
        }
    }

    let (result, ctx) = return_create_on(SpecId::HOMESTEAD, Policy::Admit, gas, code);
    assert!(ctx.offered.is_empty());
    assert_eq!(result.result, InstructionResult::OutOfGas);
}

/// A hook that fails fatally records the cause and refuses; the transaction returns the cause,
/// for a top-level creation and a nested one alike.
#[test]
fn test_a_fatal_refusal_is_the_transactions_error() {
    for spec in SCHEDULES {
        for (name, db, tx) in [
            ("top-level", db(None), create_tx(&RETURN_32, GAS_LIMIT)),
            ("nested", db(Some(create_and_record(RETURN_32))), call_tx()),
        ] {
            let (outcome, offered) = transact(db, spec, Policy::RefuseFatally, tx);

            assert_eq!(offered.len(), 1, "{spec:?}, {name}");
            match outcome {
                Err(EVMError::Custom(cause)) => assert_eq!(cause, FATAL_CAUSE),
                other => panic!("{spec:?}, {name}: expected the recorded cause, got {other:?}"),
            }
        }
    }
}
