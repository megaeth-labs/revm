//! The code-deposit admission hook on the inspected path.
//!
//! [`InspectorEvmTr::inspect_frame_run`](revm_inspector::InspectorEvmTr::inspect_frame_run)
//! commits a creation through the same `return_create` as the plain path, so the context's hook
//! is reached there too: a refused nested creation ends the same way under `inspect_tx` as under
//! `transact`, and the inspector's `create_end` sees the refusal.

use context::{
    context_interface::{
        cfg::{GasId, GasParams, StateGasSite},
        context::CodeDeposit,
        host::LoadError,
        journaled_state::AccountInfoLoad,
    },
    BlockEnv, CfgEnv, Context, ContextError, ContextSetters, ContextTr, Evm, Host, Journal,
    LocalContext, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    instructions::EthInstructions, EthPrecompiles, ExecuteEvm, MainContext, MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, CreateInputs, CreateOutcome, InstructionResult, SStoreResult,
    SelfDestructResult, StateLoad,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, TxKind, B256,
    KECCAK_EMPTY, U256,
};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{CREATE, MSTORE, PUSH0, PUSH1, PUSH4, RETURN, SSTORE, STOP},
    AccountInfo, Bytecode,
};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as context::Database>::Error;

/// Contract the transaction calls; it creates a contract that deploys 32 zero bytes.
const CONTRACT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// The output the refusing context names.
const REFUSAL: &[u8] = b"refused";
const SPEC: SpecId = SpecId::AMSTERDAM;

/// [`Context`] with an admission hook that refuses every deposit and counts them.
struct RefusingContext {
    inner: MainnetContext<Db>,
    offered: usize,
}

impl ContextTr for RefusingContext {
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

    fn admit_code_deposit(&mut self, _deposit: &CodeDeposit<'_>) -> Result<(), Bytes> {
        self.offered += 1;
        Err(Bytes::from_static(REFUSAL))
    }
}

impl ContextSetters for RefusingContext {
    fn set_tx(&mut self, tx: Self::Tx) {
        self.inner.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.inner.set_block(block);
    }
}

impl Host for RefusingContext {
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

/// Records what every creation ended with.
#[derive(Default)]
struct CreateEnds(Vec<(InstructionResult, Bytes)>);

impl Inspector<RefusingContext, EthInterpreter> for CreateEnds {
    fn create_end(
        &mut self,
        _context: &mut RefusingContext,
        _inputs: &CreateInputs,
        outcome: &mut CreateOutcome,
    ) {
        self.0
            .push((*outcome.instruction_result(), outcome.output().clone()));
    }
}

type TestEvm = Evm<
    RefusingContext,
    CreateEnds,
    EthInstructions<EthInterpreter, RefusingContext>,
    EthPrecompiles,
    handler::EthFrame<EthInterpreter>,
>;

fn evm() -> TestEvm {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    // Runs `[PUSH1 32, PUSH0, RETURN]` through `CREATE` and stores the address it pushed in slot 0.
    let code = vec![
        PUSH4, PUSH1, 32, PUSH0, RETURN, PUSH0, MSTORE, PUSH1, 4, PUSH1, 28, PUSH0, CREATE, PUSH0,
        SSTORE, STOP,
    ];
    db.insert_account_info(
        CONTRACT,
        AccountInfo::default()
            .with_nonce(1)
            .with_code(Bytecode::new_raw(code.into())),
    );
    let ctx = RefusingContext {
        inner: Context::mainnet()
            .with_db(db)
            .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SPEC)),
        offered: 0,
    };
    Evm::new_with_inspector(
        ctx,
        CreateEnds::default(),
        EthInstructions::new_mainnet_with_spec(SPEC),
        EthPrecompiles::new(SPEC),
    )
}

fn call_tx() -> TxEnv {
    TxEnv::builder()
        .caller(BENCH_CALLER)
        .kind(TxKind::Call(CONTRACT))
        .gas_limit(1_000_000)
        .build()
        .unwrap()
}

#[test]
fn test_the_inspected_path_offers_the_deposit_and_follows_the_refusal() {
    let mut plain = evm();
    let transacted = plain.transact(call_tx()).expect("the call runs");
    let mut inspected = evm();
    let inspected_outcome = inspected.inspect_tx(call_tx()).expect("the call runs");

    assert_eq!(plain.ctx.offered, 1, "the plain path offers the deposit");
    assert_eq!(inspected.ctx.offered, 1, "the inspected path offers it too");
    assert_eq!(inspected_outcome, transacted, "both paths end the same way");
    assert!(transacted.result.is_success(), "{:?}", transacted.result);
    let created = CONTRACT.create(1);
    let slot_zero = transacted.state[&CONTRACT]
        .storage
        .get(&StorageKey::ZERO)
        .map(|slot| slot.present_value)
        .unwrap_or_default();
    assert_eq!(slot_zero, U256::ZERO, "the refused CREATE pushed zero");
    assert_eq!(
        transacted
            .state
            .get(&created)
            .map_or(KECCAK_EMPTY, |account| account.info.code_hash),
        KECCAK_EMPTY
    );

    assert_eq!(
        inspected.inspector.0,
        vec![(InstructionResult::Revert, Bytes::from_static(REFUSAL))],
        "the inspector sees the creation end as the refusal"
    );
}
