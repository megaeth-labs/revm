//! The byte pool, the start charge and the history allowance, as the mainnet handler forwards
//! and settles them.
//!
//! A consumer sets the transaction's pool on the gas `tx_gas` returns; every frame then starts
//! with the share its caller's instruction forwarded, and `handle_reservoir_remaining_gas` settles
//! what it left: its remaining pool when it succeeded, the whole share when it reverted or halted.
//! A start charge a hook declared rides on the frame's input and is refunded whole when the frame
//! fails. The handler here is the mainnet one, observed through overrides that leave its behaviour
//! alone, and the frames draw the pool through instructions of the test's own, which stand in for
//! a consumer's hooks: `DRAW` draws bytes, `DECLARE` charges and declares a start charge before
//! the next start, and `LOG` charges history after drawing the frame's allowance, as a consumer's
//! write hook would.

use bytecode::opcode::{
    CALL, CREATE, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH3, REVERT, STOP,
};
use context::{
    result::{EVMError, ExecutionResult},
    Context, ContextSetters, Evm, TxEnv,
};
use context_interface::{
    cfg::{gas::InitialAndFloorGas, GasId},
    Database,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, FrameInput, GasTracker,
    Instruction, InstructionContext, InstructionExecResult, InstructionResult, StartCharge,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_handler::{
    instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr, FrameResult, Handler,
    ItemOrResult, MainContext, MainnetContext, MainnetEvm, MainnetHandler,
};
use state::{AccountInfo, Bytecode};

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<DbError>;
type Mainnet = MainnetHandler<TestEvm, TestError, EthFrame<EthInterpreter>>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 3_000_000;
/// The pool the transaction's frame starts with.
const POOL: u64 = 1_000;
/// What `DRAW` draws.
const DRAW_BYTES: u64 = 100;
/// What `DECLARE` charges and declares.
const DECLARED: StartCharge = StartCharge::new(3_520, 160);
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
    if context.interpreter.gas.record_bytes(DRAW_BYTES) {
        Ok(())
    } else {
        Err(InstructionResult::OutOfGas)
    }
}

/// What a consumer's start hook does for the start that follows: charges [`DECLARED`] on the
/// frame's gas and declares it.
const fn declare(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    let gas = &mut context.interpreter.gas;
    assert!(gas.record_history_cost(DECLARED.history));
    assert!(gas.record_bytes(DECLARED.bytes));
    gas.set_start_charge(DECLARED);
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
/// its net draw, the draw that failed, its history gas and its allowance.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Returned {
    result: InstructionResult,
    pool: u64,
    spent: i64,
    crossing: Option<u64>,
    history: i64,
    allowance: u64,
}

impl Returned {
    fn of(result: &FrameResult) -> Self {
        let gas = result.gas();
        Self {
            result: result.instruction_result(),
            pool: gas.bytes_remaining(),
            spent: gas.bytes_spent(),
            crossing: gas.bytes_crossing().map(|crossing| crossing.cost()),
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
            crossing: None,
            history: 0,
            allowance: 0,
        }
    }
}

/// The figures of a frame's input as it started: its share, its start charge and its allowance.
type Started = (u64, StartCharge, u64);

fn started(input: &FrameInput) -> Started {
    match input {
        FrameInput::Call(inputs) => (
            inputs.bytes_share,
            inputs.charged_start,
            inputs.history_allowance,
        ),
        FrameInput::Create(inputs) => (inputs.bytes_share(), inputs.charged_start(), 0),
        FrameInput::Empty => unreachable!(),
    }
}

/// The transaction's gas after the first frame settled into it: its pool, its net draw and its
/// history gas.
type Settled = (u64, i64, i64);

/// The mainnet handler with the consumer's part: a pool and a start charge set on the
/// transaction's gas, and a record of every frame's start and return.
struct PoolHandler {
    mainnet: Mainnet,
    /// The pool the transaction's gas gets, if any.
    pool: Option<u64>,
    /// Whether the transaction's gas is charged [`DECLARED`] for its frame and declares it.
    declare: bool,
    starts: Vec<Started>,
    returned: Vec<Returned>,
    settled: Option<Settled>,
}

impl PoolHandler {
    fn new(pool: Option<u64>) -> Self {
        Self {
            mainnet: Mainnet::default(),
            pool,
            declare: false,
            starts: Vec::new(),
            returned: Vec::new(),
            settled: None,
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
            gas.set_bytes_remaining(pool);
        }
        if self.declare {
            assert!(gas.record_history_cost(DECLARED.history));
            assert!(gas.record_bytes(DECLARED.bytes));
            gas.set_start_charge(DECLARED);
        }
        gas
    }

    /// The default loop, recording each frame's input as it starts and its result as it
    /// returns, before its caller settles it.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        self.starts.push(started(&first_frame_input.frame_input));
        if let ItemOrResult::Result(result) = evm.frame_init(first_frame_input)? {
            self.returned.push(Returned::of(&result));
            return Ok(result);
        }
        loop {
            let result = match evm.frame_run()? {
                ItemOrResult::Item(init) => {
                    self.starts.push(started(&init.frame_input));
                    match evm.frame_init(init)? {
                        ItemOrResult::Item(_) => continue,
                        ItemOrResult::Result(result) => result,
                    }
                }
                ItemOrResult::Result(result) => result,
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
                gas.bytes_remaining(),
                gas.bytes_spent(),
                gas.history_gas_spent(),
            )))
        });
        self.mainnet
            .post_execution(evm, exec_result, init_and_floor_gas, eip7702_refund)
    }
}

thread_local! {
    static SETTLED: core::cell::Cell<Option<Settled>> = const { core::cell::Cell::new(None) };
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

/// A transaction to [`PARENT`], holding `parent`, on the accounts every test knows.
struct Run {
    parent: Vec<u8>,
    parent_balance: u64,
    parent_nonce: u64,
    /// `call_history_allowance` in the schedule.
    allowance: u64,
}

impl Run {
    const fn new(parent: Vec<u8>) -> Self {
        Self {
            parent,
            parent_balance: 10,
            parent_nonce: 1,
            allowance: 0,
        }
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
        let ctx: TestContext = Context::mainnet()
            .with_db(self.db())
            .modify_cfg_chained(|cfg| {
                cfg.set_spec_and_mainnet_gas_params(SPEC);
                cfg.gas_params
                    .override_gas([(GasId::call_history_allowance(), allowance)]);
            });
        let mut instructions = EthInstructions::new_mainnet_with_spec(SPEC);
        instructions.insert_instruction(DRAW, Instruction::new(draw), 0);
        instructions.insert_instruction(DECLARE, Instruction::new(declare), 0);
        instructions.insert_instruction(LOG, Instruction::new(log), 0);
        let mut evm: TestEvm = Evm::new(ctx, instructions, EthPrecompiles::new(SPEC));
        evm.ctx.set_tx(
            TxEnv::builder_for_bench()
                .kind(TxKind::Call(PARENT))
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
        let result = handler.run(&mut self.evm()).expect("the transaction runs");
        handler.settled = SETTLED.with(|settled| settled.get());
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
                crossing: Some(DRAW_BYTES),
                ..Returned::ended(InstructionResult::OutOfGas, 50, 0)
            },
            Returned::stopped(50, 100),
        ]
    );
    assert_eq!(handler.settled, Some((50, 100, 0)));
}

/// A start charge declared before a start rides on the frame's input, and is refunded to the
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
            (POOL - DECLARED.bytes, DECLARED, 0),
            "{name}: the charge rides on the input"
        );
        let settled = handler.settled.expect(name);
        if fails {
            assert_eq!(settled, (POOL, 0, 0), "{name}: refunded whole");
        } else {
            let drawn = DECLARED.bytes + DRAW_BYTES;
            assert_eq!(
                settled,
                (POOL - drawn, drawn as i64, DECLARED.history as i64),
                "{name}: kept"
            );
        }
    }
}

/// The transaction's frame gets the pool as the consumer left it after the body's charges, and
/// the start charge the consumer declared, which is refunded whole when the frame reverts or
/// halts and kept when it succeeds.
#[test]
fn test_the_first_frame_gets_the_pool_and_its_start_charge() {
    for (name, parent, fails) in [
        ("stops", vec![DRAW, STOP], false),
        ("reverts", vec![DRAW, PUSH0, PUSH0, REVERT], true),
        ("halts", vec![DRAW, INVALID], true),
    ] {
        let mut handler = PoolHandler::new(Some(POOL));
        handler.declare = true;

        let (result, handler) = Run::new(parent).with(handler);

        assert_eq!(result.is_success(), name == "stops", "{name}: {result:?}");
        assert_eq!(
            handler.starts,
            [(POOL - DECLARED.bytes, DECLARED, 0)],
            "{name}"
        );
        let settled = handler.settled.expect(name);
        if fails {
            assert_eq!(settled, (POOL, 0, 0), "{name}: refunded whole");
        } else {
            let drawn = DECLARED.bytes + DRAW_BYTES;
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
