//! The byte pool, the start charge and the history allowance on the plain and the inspected path.
//!
//! `transact` runs the frames in the handler's loop and `inspect_tx` in the inspector's; both
//! build frames with `make_call_frame` and `make_create_frame` and settle them with
//! `return_result`, so a child's share, its start charge and its allowance travel the same way
//! and settle to the same figures. Each test runs a transaction both ways on the mainnet context
//! with instructions of the test's own, which stand in for a consumer's hooks, and checks that the
//! two paths end the same way; an inspector records each frame's input as it starts and its
//! outcome as it ends, before its caller settles it, and the figures are checked on the inspected
//! path (the handler's tests check them on the plain one).

use context::{result::ResultAndState, Context, Evm, TxEnv};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, ExecuteEvm, MainContext,
    MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, CallInputs, CallOutcome, CreateInputs, CreateOutcome, Instruction,
    InstructionContext, InstructionExecResult, InstructionResult, StartCharge,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{
        CALL, CREATE, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3, REVERT, STOP,
    },
    AccountInfo, Bytecode,
};

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;
type PoolEvm = Evm<Ctx, Figures, EthInstructions<EthInterpreter, Ctx>, EthPrecompiles, EthFrame>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 3_000_000;
/// What `DRAW` draws.
const DRAW_BYTES: u64 = 100;
/// What `DECLARE` charges and declares.
const DECLARED: StartCharge = StartCharge::new(3_520, 160);
/// What `LOG` charges as history.
const HISTORY_COST: u64 = 100;
/// A value call's history allowance on a chain that gives one.
const ALLOWANCE: u64 = 160;
/// The pool nobody set.
const UNLIMITED: u64 = u64::MAX;

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

/// What a consumer's start hook does for the start that follows.
const fn declare(context: InstructionContext<'_, Ctx, EthInterpreter>) -> InstructionExecResult {
    let gas = &mut context.interpreter.gas;
    assert!(gas.record_history_cost(DECLARED.history));
    assert!(gas.record_bytes(DECLARED.bytes));
    gas.set_start_charge(DECLARED);
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

/// A frame's figures as it ended, before its caller settled it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Ended {
    result: InstructionResult,
    pool: u64,
    spent: i64,
    crossing: Option<u64>,
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
            crossing: None,
            history: 0,
            allowance: 0,
            charged_start: StartCharge::new(0, 0),
        }
    }
}

/// Every frame's input as it started (share, start charge, allowance) and its outcome as it
/// ended.
#[derive(Default)]
struct Figures {
    starts: Vec<(u64, StartCharge, u64)>,
    ends: Vec<Ended>,
}

impl Figures {
    fn end(&mut self, result: InstructionResult, gas: &interpreter::Gas, charged: StartCharge) {
        self.ends.push(Ended {
            result,
            pool: gas.bytes_remaining(),
            spent: gas.bytes_spent(),
            crossing: gas.bytes_crossing().map(|crossing| crossing.cost()),
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
        None
    }

    fn create(&mut self, _context: &mut Ctx, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        self.starts
            .push((inputs.bytes_share(), inputs.charged_start(), 0));
        None
    }

    fn call_end(&mut self, _context: &mut Ctx, _inputs: &CallInputs, outcome: &mut CallOutcome) {
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
        let ctx: Ctx = Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SPEC);
            cfg.gas_params.override_gas([(
                context::context_interface::cfg::GasId::call_history_allowance(),
                allowance,
            )]);
        });
        let mut instructions = EthInstructions::new_mainnet_with_spec(SPEC);
        instructions.insert_instruction(DRAW, Instruction::new(draw), 0);
        instructions.insert_instruction(DRAIN, Instruction::new(drain), 0);
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
                crossing: Some(u64::MAX),
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
