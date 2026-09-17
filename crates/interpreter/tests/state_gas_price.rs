//! The EIP-8037 state gas pricing hook at the opcode call sites.
//!
//! Every opcode that charges or refills state gas asks [`Host::state_gas_price`] for the price of
//! the account or slot the charge lands on. These tests run the instructions against a host that
//! prices chosen sites away from the flat schedule, so each one fails if its call site reads the
//! schedule directly, and against a host whose lookup fails, which must halt the frame with
//! [`InstructionResult::FatalExternalError`].

use bytecode::{opcode::*, Bytecode};
use context_interface::{
    cfg::{GasId, GasParams, StateGasSite},
    host::LoadError,
    journaled_state::AccountInfoLoad,
};
use primitives::{
    address, hardfork::SpecId, Address, Bytes, HashMap, HashSet, Log, StorageKey, StorageValue,
    B256, KECCAK_EMPTY, U256,
};
use revm_interpreter::{
    instruction_table,
    instructions::gas_table_spec,
    interpreter::{EthInterpreter, ExtBytecode},
    FrameInput, Host, InputsImpl, InstructionResult, Interpreter, InterpreterAction, SStoreResult,
    SelfDestructResult, SharedMemory, StateLoad,
};
use state::AccountInfo;
use std::borrow::Cow;

/// Account whose code the interpreter runs.
const CONTRACT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// An account with neither code nor balance.
const EMPTY: Address = address!("0x00000000000000000000000000000000000e3970");
/// An account that is priced like `EMPTY` but is not the target of any charge.
const DECOY: Address = address!("0x00000000000000000000000000000000000dec0e");

const GAS_LIMIT: u64 = 10_000_000;

/// Prices well away from the flat Amsterdam schedule (97,920 per slot, 183,600 per account).
const PRICE_A: u64 = 500_000;
const PRICE_B: u64 = 700_000;

/// One state gas price lookup: which price, at which site.
type Lookup = (GasId, StateGasSite);

/// A host that prices state gas from a per-site table and keeps just enough state for the
/// instructions under test.
#[derive(Debug)]
struct PricedHost {
    gas_params: GasParams,
    /// Unit prices that differ from the flat schedule.
    prices: HashMap<Lookup, u64>,
    /// A lookup that fails.
    poison: Option<Lookup>,
    /// Every lookup, in order.
    lookups: Vec<Lookup>,
    /// `(original, present)` per slot; absent slots are zero.
    storage: HashMap<(Address, StorageKey), (StorageValue, StorageValue)>,
    /// Accounts that exist; every other account is empty.
    existing: HashSet<Address>,
}

impl PricedHost {
    fn new() -> Self {
        Self {
            gas_params: GasParams::new_spec(SpecId::AMSTERDAM),
            prices: HashMap::default(),
            poison: None,
            lookups: Vec::new(),
            storage: HashMap::default(),
            existing: HashSet::from_iter([CONTRACT]),
        }
    }

    fn with_price(mut self, id: GasId, site: StateGasSite, price: u64) -> Self {
        self.prices.insert((id, site), price);
        self
    }

    const fn with_poison(mut self, id: GasId, site: StateGasSite) -> Self {
        self.poison = Some((id, site));
        self
    }

    /// A slot whose committed value is `original` and whose current value is `present`.
    fn with_slot(mut self, key: StorageKey, original: u64, present: u64) -> Self {
        self.storage.insert(
            (CONTRACT, key),
            (StorageValue::from(original), StorageValue::from(present)),
        );
        self
    }
}

impl Host for PricedHost {
    fn state_gas_price(&mut self, id: GasId, site: StateGasSite) -> Option<u64> {
        self.lookups.push((id, site));
        if self.poison == Some((id, site)) {
            return None;
        }
        Some(
            self.prices
                .get(&(id, site))
                .copied()
                .unwrap_or_else(|| self.gas_params.get(id)),
        )
    }

    fn basefee(&self) -> U256 {
        U256::ZERO
    }

    fn blob_gasprice(&self) -> U256 {
        U256::ZERO
    }

    fn gas_limit(&self) -> U256 {
        U256::ZERO
    }

    fn difficulty(&self) -> U256 {
        U256::ZERO
    }

    fn prevrandao(&self) -> Option<U256> {
        None
    }

    fn block_number(&self) -> U256 {
        U256::ZERO
    }

    fn timestamp(&self) -> U256 {
        U256::ZERO
    }

    fn beneficiary(&self) -> Address {
        Address::ZERO
    }

    fn slot_num(&self) -> U256 {
        U256::ZERO
    }

    fn chain_id(&self) -> U256 {
        U256::ZERO
    }

    fn effective_gas_price(&self) -> U256 {
        U256::ZERO
    }

    fn caller(&self) -> Address {
        Address::ZERO
    }

    fn blob_hash(&self, _number: usize) -> Option<U256> {
        None
    }

    fn max_initcode_size(&self) -> usize {
        usize::MAX
    }

    fn gas_params(&self) -> &GasParams {
        &self.gas_params
    }

    fn is_amsterdam_eip8037_enabled(&self) -> bool {
        true
    }

    fn block_hash(&mut self, _number: u64) -> Option<B256> {
        None
    }

    /// Every selfdestruct moves value into an account that does not exist, the case that tops
    /// the beneficiary up.
    fn selfdestruct(
        &mut self,
        _address: Address,
        _target: Address,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, LoadError> {
        let result = SelfDestructResult {
            had_value: true,
            target_exists: false,
            previously_destroyed: false,
        };
        Ok(StateLoad::new(result, false))
    }

    fn log(&mut self, _log: Log) {}

    fn sstore_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, LoadError> {
        let slot = self.storage.entry((address, key)).or_default();
        let result = SStoreResult {
            original_value: slot.0,
            present_value: slot.1,
            new_value: value,
        };
        slot.1 = value;
        Ok(StateLoad::new(result, false))
    }

    fn sload_skip_cold_load(
        &mut self,
        address: Address,
        key: StorageKey,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, LoadError> {
        let present = self
            .storage
            .get(&(address, key))
            .map_or(U256::ZERO, |s| s.1);
        Ok(StateLoad::new(present, false))
    }

    fn tstore(&mut self, _address: Address, _key: StorageKey, _value: StorageValue) {}

    fn tload(&mut self, _address: Address, _key: StorageKey) -> StorageValue {
        StorageValue::ZERO
    }

    fn load_account_info_skip_cold_load(
        &mut self,
        address: Address,
        _load_code: bool,
        _skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, LoadError> {
        Ok(AccountInfoLoad {
            account: Cow::Owned(AccountInfo::default()),
            is_cold: false,
            is_empty: !self.existing.contains(&address),
        })
    }
}

/// Runs `code` as [`CONTRACT`] until it returns or asks for a new frame.
fn run(code: Vec<u8>, host: &mut PricedHost) -> (Interpreter<EthInterpreter>, InterpreterAction) {
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(Bytecode::new_raw(Bytes::from(code))),
        InputsImpl {
            target_address: CONTRACT,
            ..Default::default()
        },
        false,
        SpecId::AMSTERDAM,
        GAS_LIMIT,
    );
    let table = instruction_table::<EthInterpreter, PricedHost>();
    let gas = gas_table_spec(SpecId::AMSTERDAM);
    let action = interpreter.run_plain(&table, &gas, host);
    (interpreter, action)
}

fn push_address(code: &mut Vec<u8>, address: Address) {
    code.push(PUSH20);
    code.extend_from_slice(address.as_slice());
}

/// `SSTORE(key, value)`.
fn sstore(code: &mut Vec<u8>, key: u8, value: u8) {
    code.extend_from_slice(&[PUSH1, value, PUSH1, key, SSTORE]);
}

/// `CALL(100_000, to, 1, 0, 0, 0, 0)`: a value transfer with no data.
fn value_call(to: Address) -> Vec<u8> {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH1, 1];
    push_address(&mut code, to);
    code.extend_from_slice(&[PUSH3, 0x01, 0x86, 0xa0, CALL, STOP]);
    code
}

/// The two contract-creating opcodes, each deploying the empty init code from [`CONTRACT`] at
/// nonce 0.
#[derive(Clone, Copy)]
enum CreateOpcode {
    Create,
    Create2,
}

impl CreateOpcode {
    /// `CREATE(0, 0, 0)` or `CREATE2(0, 0, 0, 0)`.
    fn code(self) -> Vec<u8> {
        match self {
            Self::Create => vec![PUSH0, PUSH0, PUSH0, CREATE, STOP],
            Self::Create2 => vec![PUSH0, PUSH0, PUSH0, PUSH0, CREATE2, STOP],
        }
    }

    /// The address the empty init code is deployed to.
    fn created_address(self) -> Address {
        match self {
            Self::Create => CONTRACT.create(0),
            Self::Create2 => CONTRACT.create2(B256::ZERO, KECCAK_EMPTY),
        }
    }
}

/// `SELFDESTRUCT(beneficiary)`.
fn selfdestruct(beneficiary: Address) -> Vec<u8> {
    let mut code = Vec::new();
    push_address(&mut code, beneficiary);
    code.push(SELFDESTRUCT);
    code
}

fn slot(key: u8) -> StateGasSite {
    StateGasSite::slot(CONTRACT, StorageKey::from(key))
}

fn assert_halted_fatal(action: &InterpreterAction) {
    match action {
        InterpreterAction::Return(result) => {
            assert_eq!(result.result, InstructionResult::FatalExternalError)
        }
        other => panic!("expected a fatal halt, got {other:?}"),
    }
}

#[test]
fn test_sstore_charge_is_priced_per_slot() {
    let mut code = Vec::new();
    sstore(&mut code, 1, 1);
    sstore(&mut code, 2, 1);
    code.push(STOP);
    let mut host = PricedHost::new()
        .with_price(GasId::sstore_set_state_gas(), slot(1), PRICE_A)
        .with_price(GasId::sstore_set_state_gas(), slot(2), PRICE_B);

    let (interpreter, action) = run(code, &mut host);

    assert!(action.is_return());
    assert_eq!(
        interpreter.gas.state_gas_spent(),
        (PRICE_A + PRICE_B) as i64,
        "each new slot pays its own price"
    );
    assert_eq!(
        host.lookups,
        [
            (GasId::sstore_set_state_gas(), slot(1)),
            (GasId::sstore_set_state_gas(), slot(2))
        ]
    );
}

#[test]
fn test_sstore_restoration_refills_the_slot_price() {
    // The 0 -> 1 write was paid for earlier in the transaction; restoring the slot to zero
    // refills what that write cost at this slot.
    let mut code = Vec::new();
    sstore(&mut code, 1, 0);
    code.push(STOP);
    let mut host = PricedHost::new()
        .with_slot(StorageKey::from(1), 0, 1)
        .with_price(GasId::sstore_set_state_gas(), slot(1), PRICE_A);

    let (interpreter, action) = run(code, &mut host);

    assert!(action.is_return());
    assert_eq!(interpreter.gas.state_gas_spent(), -(PRICE_A as i64));
    assert_eq!(interpreter.gas.reservoir(), PRICE_A);
    assert_eq!(host.lookups, [(GasId::sstore_set_state_gas(), slot(1))]);
}

#[test]
fn test_sstore_charge_lookup_failure_halts_fatal() {
    let mut code = Vec::new();
    sstore(&mut code, 1, 1);
    code.push(STOP);
    let mut host = PricedHost::new().with_poison(GasId::sstore_set_state_gas(), slot(1));

    let (interpreter, action) = run(code, &mut host);

    assert_halted_fatal(&action);
    assert_eq!(interpreter.gas.state_gas_spent(), 0);
    assert_eq!(host.lookups, [(GasId::sstore_set_state_gas(), slot(1))]);
}

#[test]
fn test_sstore_restoration_lookup_failure_halts_fatal() {
    let mut code = Vec::new();
    sstore(&mut code, 1, 0);
    code.push(STOP);
    let mut host = PricedHost::new()
        .with_slot(StorageKey::from(1), 0, 1)
        .with_poison(GasId::sstore_set_state_gas(), slot(1));

    let (interpreter, action) = run(code, &mut host);

    assert_halted_fatal(&action);
    assert_eq!(interpreter.gas.reservoir(), 0, "nothing was refilled");
    assert_eq!(host.lookups, [(GasId::sstore_set_state_gas(), slot(1))]);
}

#[test]
fn test_call_new_account_charge_is_priced_per_callee() {
    let callee = StateGasSite::account(EMPTY);
    let mut host = PricedHost::new()
        .with_price(GasId::new_account_state_gas(), callee, PRICE_A)
        .with_price(
            GasId::new_account_state_gas(),
            StateGasSite::account(DECOY),
            PRICE_B,
        );

    let (interpreter, action) = run(value_call(EMPTY), &mut host);

    let InterpreterAction::NewFrame(FrameInput::Call(inputs)) = action else {
        panic!("expected a call frame, got {action:?}");
    };
    assert!(inputs.charged_new_account_state_gas);
    assert_eq!(
        inputs.target_address, EMPTY,
        "the refund re-prices this site"
    );
    assert_eq!(interpreter.gas.state_gas_spent(), PRICE_A as i64);
    assert_eq!(host.lookups, [(GasId::new_account_state_gas(), callee)]);
}

#[test]
fn test_call_new_account_lookup_failure_halts_fatal() {
    let callee = StateGasSite::account(EMPTY);
    let mut host = PricedHost::new().with_poison(GasId::new_account_state_gas(), callee);

    let (interpreter, action) = run(value_call(EMPTY), &mut host);

    assert_halted_fatal(&action);
    assert_eq!(interpreter.gas.state_gas_spent(), 0);
    assert_eq!(host.lookups, [(GasId::new_account_state_gas(), callee)]);
}

fn assert_create_priced_per_created_address(opcode: CreateOpcode) {
    let created = StateGasSite::account(opcode.created_address());
    let mut host = PricedHost::new()
        .with_price(GasId::create_state_gas(), created, PRICE_A)
        .with_price(
            GasId::create_state_gas(),
            StateGasSite::account(DECOY),
            PRICE_B,
        );

    let (interpreter, action) = run(opcode.code(), &mut host);

    let InterpreterAction::NewFrame(FrameInput::Create(inputs)) = action else {
        panic!("expected a create frame, got {action:?}");
    };
    assert!(inputs.charged_create_state_gas());
    assert_eq!(
        inputs.charged_state_gas_address(),
        created.address,
        "the refund re-prices this site"
    );
    assert_eq!(interpreter.gas.state_gas_spent(), PRICE_A as i64);
    assert_eq!(host.lookups, [(GasId::create_state_gas(), created)]);
}

#[test]
fn test_create_charge_is_priced_per_created_address() {
    assert_create_priced_per_created_address(CreateOpcode::Create);
}

#[test]
fn test_create2_charge_is_priced_per_created_address() {
    assert_create_priced_per_created_address(CreateOpcode::Create2);
}

fn assert_create_lookup_failure_halts_fatal(opcode: CreateOpcode) {
    let created = StateGasSite::account(opcode.created_address());
    let mut host = PricedHost::new().with_poison(GasId::create_state_gas(), created);

    let (interpreter, action) = run(opcode.code(), &mut host);

    assert_halted_fatal(&action);
    assert_eq!(interpreter.gas.state_gas_spent(), 0);
    assert_eq!(host.lookups, [(GasId::create_state_gas(), created)]);
}

#[test]
fn test_create_lookup_failure_halts_fatal() {
    assert_create_lookup_failure_halts_fatal(CreateOpcode::Create);
}

#[test]
fn test_create2_lookup_failure_halts_fatal() {
    assert_create_lookup_failure_halts_fatal(CreateOpcode::Create2);
}

#[test]
fn test_selfdestruct_top_up_is_priced_per_beneficiary() {
    let beneficiary = StateGasSite::account(EMPTY);
    let mut host = PricedHost::new()
        .with_price(GasId::new_account_state_gas(), beneficiary, PRICE_A)
        .with_price(
            GasId::new_account_state_gas(),
            StateGasSite::account(CONTRACT),
            PRICE_B,
        );

    let (interpreter, action) = run(selfdestruct(EMPTY), &mut host);

    let InterpreterAction::Return(result) = action else {
        panic!("expected a return, got {action:?}");
    };
    assert_eq!(result.result, InstructionResult::SelfDestruct);
    assert_eq!(interpreter.gas.state_gas_spent(), PRICE_A as i64);
    assert_eq!(
        host.lookups,
        [(GasId::new_account_state_gas(), beneficiary)]
    );
}

#[test]
fn test_selfdestruct_lookup_failure_halts_fatal() {
    let beneficiary = StateGasSite::account(EMPTY);
    let mut host = PricedHost::new().with_poison(GasId::new_account_state_gas(), beneficiary);

    let (interpreter, action) = run(selfdestruct(EMPTY), &mut host);

    assert_halted_fatal(&action);
    assert_eq!(interpreter.gas.state_gas_spent(), 0);
    assert_eq!(
        host.lookups,
        [(GasId::new_account_state_gas(), beneficiary)]
    );
}
