//! An outcome an inspector builds to answer a start settles its reservoir as built, on top of
//! what the caller kept when it forwarded its share.
//!
//! The caller's start takes the schedule's share of its reservoir out of its tracker, and the
//! answer's reservoir is added back to what the caller kept, as a frame's is. An answer built on
//! `Gas::new` carries no reservoir: with the whole reservoir forwarded, which every schedule does,
//! the caller is left with none, as upstream's settlement, which put the answer's reservoir in
//! place of the caller's, leaves it; with 98% forwarded, the caller is left with the 2% it kept.
//! An answer that carries the input's reservoir gives the caller back what it forwarded.
//!
//! A creation's answer settles through the same arm as a call's: the caller also paid the
//! creation's upfront state charge before the forward, which an answer that copies the input's
//! charge gives back and one built with `CreateOutcome::new` does not.

use context::{context_interface::cfg::GasId, Context, Evm, TxEnv};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, MainContext, MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome, Gas,
    InstructionResult, InterpreterResult, PoolAmounts,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{CALL, CREATE, POP, PUSH0, PUSH1, PUSH20, PUSH3, SSTORE, STOP},
    AccountInfo, Bytecode,
};

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;

const SPEC: SpecId = SpecId::AMSTERDAM;
const GAS_LIMIT: u64 = 3_000_000;
/// The reservoir the transaction starts with: the regular cap is this far below [`GAS_LIMIT`].
const RESERVOIR: u64 = 1_000_000;
/// State gas of a new storage slot on Amsterdam's schedule, which [`PARENT`] pays.
const SLOT: u64 = 97_920;
/// What [`PARENT`] holds when it calls [`OK`].
const BEFORE_THE_CALL: u64 = RESERVOIR - SLOT;
/// Its share at 98: all but the caller's 2%, rounded down (18,041.6 → 18,041).
const SHARE: u64 = 884_039;

/// The state charge a creation's start pays for its account on Amsterdam's schedule, before the
/// forward.
const ACCOUNT: u64 = 183_600;
/// What [`PARENT`] holds when it creates, once the creation's charge is paid.
const BEFORE_THE_CREATION: u64 = BEFORE_THE_CALL - ACCOUNT;
/// Its share at 98: all but the caller's 2%, rounded down (14,369.6 → 14,369).
const CREATION_SHARE: u64 = 704_111;

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
    /// Answers it with a revert that carries the input's reservoir and pool shares.
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
    /// The gas of the answer to a start of `gas_limit` given `reservoir` and `pools`, or `None`
    /// to let the start run.
    const fn gas(&mut self, gas_limit: u64, reservoir: u64, pools: PoolAmounts) -> Option<Gas> {
        self.given = Some(reservoir);
        match self.answer {
            Answer::Runs => None,
            Answer::AsBuilt => Some(Gas::new(gas_limit)),
            Answer::WithTheShares => {
                let mut gas = Gas::new_with_regular_gas_and_reservoir(gas_limit, reservoir);
                gas.tracker_mut().set_pools_remaining(pools);
                Some(gas)
            }
        }
    }
}

impl Inspector<Ctx, EthInterpreter> for Answering {
    fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        if inputs.target_address != OK {
            return None;
        }
        let gas = self.gas(inputs.gas_limit, inputs.reservoir, inputs.pool_shares)?;
        Some(CallOutcome::new(
            InterpreterResult::new(InstructionResult::Revert, Bytes::new(), gas),
            inputs.return_memory_offset.clone(),
        ))
    }

    fn create(&mut self, _: &mut Ctx, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        let gas = self.gas(inputs.gas_limit(), inputs.reservoir(), inputs.pool_shares())?;
        let mut outcome = CreateOutcome::new(
            InterpreterResult::new(InstructionResult::Revert, Bytes::new(), gas),
            None,
        );
        if let Answer::WithTheShares = self.answer {
            outcome.charged_create_state_gas = inputs.charged_create_state_gas();
            outcome.charged_state_gas_address = inputs.charged_state_gas_address();
            outcome.charged_start = inputs.charged_start();
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
/// `percent` per cent of a caller's reservoir forwarded: the schedule's `reservoir_kept_percent`
/// is the rest.
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
        if let Some(percent) = percent {
            cfg.gas_params
                .override_gas([(GasId::reservoir_kept_percent(), 100 - percent)]);
        }
    });
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
        // Upstream's settlement gives the caller the answer's empty reservoir in place of its
        // own; the whole forward left the caller nothing to add it to, so the same.
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

/// A creation's answer settles as a call's: as built, the caller keeps its kept part and the
/// creation's charge; with the shares and the input's charges, the caller gets back what it held
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
