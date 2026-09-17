//! The history gas counter through the mainnet handler.
//!
//! Nothing in this workspace books history gas except the code deposit, and only when the schedule
//! prices it. So the interpreter here gets three extra opcodes that do what a chain pricing history
//! bytes does at its own charge sites: book a charge as history ([`Gas::record_history_cost`]),
//! take one back ([`Gas::refill_history`]), and, as the twin a history charge must behave like,
//! book the same amount as state gas ([`Gas::record_state_cost`]).
//!
//! The handler is the mainnet one, observed through two overrides that leave its behaviour alone:
//! each frame's gas is recorded as the frame returns, before its caller settles it, and the
//! transaction-level gas is recorded once the first frame has been settled into it.
//!
//! [`Gas::record_history_cost`]: interpreter::Gas::record_history_cost
//! [`Gas::refill_history`]: interpreter::Gas::refill_history
//! [`Gas::record_state_cost`]: interpreter::Gas::record_state_cost

use bytecode::opcode::{
    CALL, CODECOPY, CREATE, DELEGATECALL, INVALID, MSTORE, POP, PUSH0, PUSH1, PUSH20, PUSH32,
    PUSH4, RETURN, REVERT, SSTORE, STOP,
};
use context::{
    result::{EVMError, ExecutionResult, HaltReason, OutOfGasError, Output},
    Context, ContextSetters, Evm, TxEnv,
};
use context_interface::{cfg::GasId, result::ResultGas, Cfg, ContextTr, Database};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, GasTracker, Instruction,
    InstructionContext, InstructionExecResult, InstructionResult,
};
use primitives::{address, eip8037, hardfork::SpecId, Address, Bytes, TxKind, KECCAK_EMPTY, U256};
use revm_handler::{
    instructions::EthInstructions, post_execution, EthPrecompiles, EvmTr, ExecuteEvm, FrameResult,
    Handler, ItemOrResult, MainContext, MainnetContext, MainnetEvm,
};
use state::{AccountInfo, Bytecode, EvmState};
use std::cell::Cell;

type Db = CacheDB<EmptyDB>;
type DbError = <Db as Database>::Error;
type TestContext = MainnetContext<Db>;
type TestEvm = MainnetEvm<TestContext>;
type TestError = EVMError<DbError>;

/// Contract the transaction calls.
const CONTRACT: Address = address!("0x000000000000000000000000000000000000c0de");
/// Contract [`CONTRACT`] calls.
const CHILD: Address = address!("0x000000000000000000000000000000000000c41d");
/// Contract [`CONTRACT`] calls after [`CHILD`].
const SIBLING: Address = address!("0x000000000000000000000000000000000000c41e");

/// Books the popped amount as history gas; out of gas if the budget cannot pay it.
const CHARGE_HISTORY: u8 = 0x0c;
/// Refills the popped amount of history gas.
const REFILL_HISTORY: u8 = 0x0d;
/// Books the popped amount as state gas; out of gas if the budget cannot pay it.
const CHARGE_STATE: u8 = 0x0e;

/// Regular gas budget of a transaction; the gas limit above it is the reservoir.
const REGULAR_CAP: u64 = 2_000_000;
/// The reservoir a call transaction starts with.
const RESERVOIR: u64 = 100_000;
/// Gas [`CONTRACT`] forwards to [`CHILD`].
const CHILD_GAS: u64 = 500_000;

/// History gas the frames under test charge.
const HISTORY: u64 = 50_000;
/// A history charge larger than [`RESERVOIR`], which spills onto regular gas.
const LARGE_HISTORY: u64 = 150_000;

/// State gas for a 0→x storage write on the flat Amsterdam schedule.
const SSTORE_SET: u64 = eip8037::SSTORE_SET_BYTES * eip8037::CPSB_GLAMSTERDAM;
/// State gas for creating an account on the flat Amsterdam schedule.
const NEW_ACCOUNT: u64 = eip8037::NEW_ACCOUNT_BYTES * eip8037::CPSB_GLAMSTERDAM;

/// Length of the code the test initcode deploys.
const CODE_LEN: u64 = 32;
/// History gas per deposited byte in the schedule that prices it.
const HISTORY_PER_BYTE: u64 = 1_000;
/// History gas for depositing [`CODE_LEN`] bytes at [`HISTORY_PER_BYTE`], signed like the counter.
const HISTORY_FOR_CODE: i64 = (CODE_LEN * HISTORY_PER_BYTE) as i64;
/// State gas for depositing [`CODE_LEN`] bytes on the flat Amsterdam schedule.
const CODE_STATE: u64 = CODE_LEN * eip8037::CODE_DEPOSIT_PER_BYTE * eip8037::CPSB_GLAMSTERDAM;

fn charge_history(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    let amount = context.interpreter.stack.pop()?.saturating_to();
    if !context.interpreter.gas.record_history_cost(amount) {
        return Err(InstructionResult::OutOfGas);
    }
    Ok(())
}

fn refill_history(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    let amount = context.interpreter.stack.pop()?.saturating_to();
    context.interpreter.gas.refill_history(amount);
    Ok(())
}

fn charge_state(
    context: InstructionContext<'_, TestContext, EthInterpreter>,
) -> InstructionExecResult {
    let amount = context.interpreter.stack.pop()?.saturating_to();
    if !context.interpreter.gas.record_state_cost(amount) {
        return Err(InstructionResult::OutOfGas);
    }
    Ok(())
}

/// Bytecode under construction.
#[derive(Default)]
struct Code(Vec<u8>);

impl Code {
    fn op(mut self, op: u8) -> Self {
        self.0.push(op);
        self
    }

    /// Pushes `value` as a four-byte word, so the gas the code costs does not depend on it.
    fn push(mut self, value: u64) -> Self {
        let value = u32::try_from(value).expect("test amounts fit in four bytes");
        self.0.push(PUSH4);
        self.0.extend_from_slice(&value.to_be_bytes());
        self
    }

    fn charge_history(self, amount: u64) -> Self {
        self.push(amount).op(CHARGE_HISTORY)
    }

    fn refill_history(self, amount: u64) -> Self {
        self.push(amount).op(REFILL_HISTORY)
    }

    fn charge_state(self, amount: u64) -> Self {
        self.push(amount).op(CHARGE_STATE)
    }

    /// Writes `value` to slot zero.
    fn sstore(self, value: u64) -> Self {
        self.push(value).push(0).op(SSTORE)
    }

    /// `CALL(CHILD_GAS, to, 0, 0, 0, 0, 0)`, dropping the success flag.
    fn call(self, to: Address) -> Self {
        self.op(PUSH0).call_code_at(to, CALL)
    }

    /// `DELEGATECALL(CHILD_GAS, to, 0, 0, 0, 0)`, dropping the success flag.
    fn delegatecall(self, to: Address) -> Self {
        self.call_code_at(to, DELEGATECALL)
    }

    /// Pushes empty argument and return buffers, `to` and [`CHILD_GAS`], then runs `call`.
    fn call_code_at(mut self, to: Address, call: u8) -> Self {
        self = self.op(PUSH0).op(PUSH0).op(PUSH0).op(PUSH0);
        self.0.push(PUSH20);
        self.0.extend_from_slice(to.as_slice());
        self.push(CHILD_GAS).op(call).op(POP)
    }

    /// `CREATE` with [`initcode`], which this code carries after its last instruction.
    fn create_and_stop(self) -> Self {
        let initcode = initcode();
        let len = initcode.len() as u64;
        let with_offset = |offset| {
            Code(self.0.clone())
                .push(len)
                .push(offset)
                .op(PUSH0)
                .op(CODECOPY)
                .push(len)
                .op(PUSH0)
                .op(PUSH0)
                .op(CREATE)
                .op(POP)
                .op(STOP)
        };
        // Four-byte pushes keep the prefix length independent of the offset it carries.
        let offset = with_offset(0).0.len() as u64;
        let mut code = with_offset(offset);
        code.0.extend_from_slice(&initcode);
        code
    }

    fn revert(self) -> Self {
        self.op(PUSH0).op(PUSH0).op(REVERT)
    }

    fn build(self) -> Bytecode {
        Bytecode::new_raw(Bytes::from(self.0))
    }
}

/// Initcode that deploys [`CODE_LEN`] `JUMPDEST` bytes.
fn initcode() -> Vec<u8> {
    let mut code = vec![PUSH32];
    code.extend_from_slice(&[0x5b; CODE_LEN as usize]);
    code.extend_from_slice(&[PUSH0, MSTORE, PUSH1, CODE_LEN as u8, PUSH0, RETURN]);
    code
}

/// A frame's gas as it returned, before its caller settled it.
#[derive(Clone, Copy, Debug)]
struct Returned {
    result: InstructionResult,
    gas: GasTracker,
}

/// The mainnet handler, recording gas as frames return and settle.
#[derive(Default)]
struct Observer {
    returned: Vec<Returned>,
    settled: Cell<Option<GasTracker>>,
}

impl Observer {
    fn record(&mut self, result: &FrameResult) {
        self.returned.push(Returned {
            result: result.instruction_result(),
            gas: *result.gas().tracker(),
        });
    }
}

impl Handler for Observer {
    type Evm = TestEvm;
    type Error = TestError;
    type HaltReason = HaltReason;

    /// The default loop, recording each frame result before it goes back to its caller.
    fn run_exec_loop(
        &mut self,
        evm: &mut TestEvm,
        first_frame_input: FrameInit,
    ) -> Result<FrameResult, TestError> {
        if let ItemOrResult::Result(result) = evm.frame_init(first_frame_input)? {
            self.record(&result);
            return Ok(result);
        }
        loop {
            let result = match evm.frame_run()? {
                ItemOrResult::Item(init) => match evm.frame_init(init)? {
                    ItemOrResult::Item(_) => continue,
                    ItemOrResult::Result(result) => result,
                },
                ItemOrResult::Result(result) => result,
            };
            self.record(&result);
            if let Some(result) = evm.frame_return_result(result)? {
                return Ok(result);
            }
        }
    }

    /// Records the settled transaction-level gas, then refunds as the default does.
    fn refund(
        &self,
        evm: &mut TestEvm,
        exec_result: &mut FrameResult,
        eip7702_refund: i64,
    ) -> Result<(), TestError> {
        self.settled.set(Some(*exec_result.gas().tracker()));
        post_execution::refund(
            evm.ctx().cfg().gas_params(),
            exec_result.gas_mut(),
            eip7702_refund,
        );
        Ok(())
    }
}

/// What the observed handler saw of one transaction.
struct Run {
    result: ExecutionResult,
    state: EvmState,
    /// Every frame's gas as it returned, innermost first.
    returned: Vec<Returned>,
    /// The transaction-level gas once the first frame settled into it.
    settled: GasTracker,
}

impl Run {
    const fn gas(&self) -> &ResultGas {
        self.result.gas()
    }

    const fn total(&self) -> u64 {
        self.gas().total_gas_spent()
    }

    /// The one frame [`CONTRACT`] called or created, as it returned.
    fn child(&self) -> Returned {
        assert_eq!(self.returned.len(), 2, "one child and the top frame");
        self.returned[0]
    }
}

/// A context on `spec` whose schedule charges `history_per_byte` for deposited code.
fn context(db: Db, spec: SpecId, history_per_byte: u64) -> TestContext {
    Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(spec);
        cfg.tx_gas_limit_cap = Some(REGULAR_CAP);
        cfg.gas_params
            .override_gas([(GasId::code_deposit_history_gas(), history_per_byte)]);
    })
}

/// An Amsterdam context whose schedule charges `history_per_byte` for deposited code.
fn amsterdam(db: Db, history_per_byte: u64) -> TestContext {
    context(db, SpecId::AMSTERDAM, history_per_byte)
}

fn run(ctx: TestContext, tx: TxEnv) -> Run {
    let spec = ctx.cfg.spec;
    let mut instructions = EthInstructions::new_mainnet_with_spec(spec);
    instructions.insert_instruction(CHARGE_HISTORY, Instruction::new(charge_history), 0);
    instructions.insert_instruction(REFILL_HISTORY, Instruction::new(refill_history), 0);
    instructions.insert_instruction(CHARGE_STATE, Instruction::new(charge_state), 0);
    let mut evm: TestEvm = Evm::new(ctx, instructions, EthPrecompiles::new(spec));
    evm.ctx.set_tx(tx);

    let mut observer = Observer::default();
    let result = observer.run(&mut evm).expect("transaction runs");
    let state = evm.finalize();
    Run {
        result,
        state,
        returned: observer.returned,
        settled: observer.settled.get().expect("post-execution ran"),
    }
}

/// A funded sender, [`CONTRACT`] holding `contract` and [`CHILD`] holding `child`.
fn db(contract: Code, child: Code) -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    for (address, code) in [(CONTRACT, contract), (CHILD, child)] {
        insert_code(&mut db, address, code);
    }
    db
}

/// Puts an account holding `code` at `address`.
fn insert_code(db: &mut Db, address: Address, code: Code) {
    let code = code.build();
    db.insert_account_info(
        address,
        AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
    );
}

fn tx(kind: TxKind, data: Vec<u8>, gas_limit: u64) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(kind)
        .data(Bytes::from(data))
        .gas_price(0)
        .gas_limit(gas_limit)
        .build_fill()
}

/// Runs a call to [`CONTRACT`] that starts with [`RESERVOIR`].
fn call(contract: Code, child: Code) -> Run {
    call_in(db(contract, child))
}

/// Runs a call to [`CONTRACT`] that starts with [`RESERVOIR`]. [`CONTRACT`] DELEGATECALLs [`CHILD`]
/// and then [`SIBLING`], which hold `siblings`.
fn call_siblings(siblings: [Code; 2]) -> Run {
    let [child, sibling] = siblings;
    let contract = Code::default()
        .delegatecall(CHILD)
        .delegatecall(SIBLING)
        .op(STOP);
    let mut db = db(contract, child);
    insert_code(&mut db, SIBLING, sibling);
    call_in(db)
}

/// Runs a call to [`CONTRACT`] in `db` that starts with [`RESERVOIR`].
fn call_in(db: Db) -> Run {
    run(
        amsterdam(db, 0),
        tx(TxKind::Call(CONTRACT), Vec::new(), REGULAR_CAP + RESERVOIR),
    )
}

/// `reservoir + state_spent + history_spent − spilled`: what a rollback sets the reservoir to,
/// which must be the reservoir the frame inherited.
const fn unwound_reservoir(gas: &GasTracker) -> i64 {
    gas.reservoir() as i64 + gas.state_gas_spent() + gas.history_gas_spent()
        - gas.state_gas_spilled() as i64
}

/// `(reservoir, state_spent, history_spent, spilled)`.
const fn counters(gas: &GasTracker) -> (u64, i64, i64, u64) {
    (
        gas.reservoir(),
        gas.state_gas_spent(),
        gas.history_gas_spent(),
        gas.state_gas_spilled(),
    )
}

/// A history charge moves the budget exactly as a state charge of the same amount does, from the
/// reservoir first and then from regular gas, and lands on the history counter only.
#[test]
fn test_a_history_charge_draws_like_a_state_charge_and_is_booked_as_history() {
    let baseline = call(Code::default().charge_history(0).op(STOP), Code::default());
    assert_eq!(baseline.settled.reservoir(), RESERVOIR);

    // (amount, reservoir left, spilled onto regular gas)
    for (amount, reservoir, spilled) in [(HISTORY, 50_000, 0), (LARGE_HISTORY, 0, 50_000)] {
        let history = call(
            Code::default().charge_history(amount).op(STOP),
            Code::default(),
        );
        let state = call(
            Code::default().charge_state(amount).op(STOP),
            Code::default(),
        );

        assert_eq!(
            counters(&history.settled),
            (reservoir, 0, amount as i64, spilled),
            "history charge of {amount}"
        );
        assert_eq!(
            history.settled.remaining(),
            baseline.settled.remaining() - spilled
        );
        assert_eq!(history.total(), baseline.total() + amount);

        assert_eq!(
            counters(&state.settled),
            (reservoir, amount as i64, 0, spilled),
            "state charge of {amount}"
        );
        assert_eq!(state.settled.remaining(), history.settled.remaining());
        assert_eq!(state.total(), history.total());
    }
}

/// A reverting frame gives back everything it charged, state and history together, in either
/// order: the parent sees the reservoir it lent and the regular gas the charges spilled.
#[test]
fn test_a_reverting_frame_returns_its_state_and_history_charges() {
    let history_last = |amount| Code::default().sstore(1).charge_history(amount).revert();
    let history_first = |amount| Code::default().charge_history(amount).sstore(1).revert();

    for child in [history_last, history_first] {
        let charged = call(Code::default().call(CHILD).op(STOP), child(HISTORY));
        let uncharged = call(Code::default().call(CHILD).op(STOP), child(0));

        let returned = charged.child();
        assert_eq!(returned.result, InstructionResult::Revert);
        // 97,920 + 50,000 against a 100,000 reservoir: 47,920 spilled.
        assert_eq!(
            counters(&returned.gas),
            (0, SSTORE_SET as i64, HISTORY as i64, 47_920)
        );
        assert_eq!(unwound_reservoir(&returned.gas), RESERVOIR as i64);
        assert_eq!(
            returned.gas.remaining(),
            uncharged.child().gas.remaining() - 47_920
        );

        assert_eq!(counters(&charged.settled), (RESERVOIR, 0, 0, 0));
        assert_eq!(charged.total(), uncharged.total());
        assert_eq!(charged.gas().state_gas_spent_final(), 0);
    }
}

/// A reverting frame that took back its caller's state charge after its own history charge
/// spilled still lands on the reservoir it inherited. Its net state gas is negative while the
/// reservoir it holds is smaller than that, so the unwind must not clamp part-way.
#[test]
fn test_a_reverting_frame_that_refilled_its_callers_state_gas_returns_the_inherited_reservoir() {
    // The child runs in the caller's storage: it clears the slot the caller set, which refills
    // the caller's 0→x charge.
    let run = |amount| {
        call(
            Code::default().sstore(1).delegatecall(CHILD).op(STOP),
            Code::default().charge_history(amount).sstore(0).revert(),
        )
    };
    let charged = run(HISTORY);
    let uncharged = run(0);

    // The caller's write leaves 2,080 of the reservoir. The child's history charge takes those
    // and spills 47,920; clearing the slot credits the 47,920 back to regular gas and 50,000 to
    // the reservoir.
    let inherited = RESERVOIR - SSTORE_SET;
    let returned = charged.child();
    assert_eq!(returned.result, InstructionResult::Revert);
    assert_eq!(
        counters(&returned.gas),
        (50_000, -(SSTORE_SET as i64), HISTORY as i64, 0)
    );
    assert_eq!(unwound_reservoir(&returned.gas), inherited as i64);

    assert_eq!(
        counters(&charged.settled),
        (inherited, SSTORE_SET as i64, 0, 0)
    );
    assert_eq!(counters(&uncharged.settled), counters(&charged.settled));
    assert_eq!(charged.total(), uncharged.total());
}

/// State and history refills credit one shared spill counter, so a refill of one kind can return
/// regular gas a charge of the other kind spilled. A frame that does this and then reverts still
/// hands its caller the reservoir it inherited and the regular gas it was lent.
#[test]
fn test_a_reverting_frame_whose_refill_took_the_other_kinds_spill_returns_the_inherited_reservoir()
{
    // (child, the same child charging nothing, the child's counters as it reverts)
    let rows = [
        // The write leaves 2,080 of the reservoir. The history charge takes them and spills
        // 47,920. Clearing the slot refills 97,920 state gas: the 47,920 the history charge
        // spilled go back to regular gas and 50,000 to the reservoir.
        (
            Code::default()
                .sstore(1)
                .charge_history(HISTORY)
                .sstore(0)
                .revert(),
            Code::default()
                .sstore(1)
                .charge_history(0)
                .sstore(0)
                .revert(),
            (50_000, 0, HISTORY as i64, 0),
        ),
        // The history charge leaves 50,000 of the reservoir. The write takes them and spills
        // 47,920. Refilling 20,000 history gas returns 20,000 of the write's spill to regular gas.
        (
            Code::default()
                .charge_history(HISTORY)
                .sstore(1)
                .refill_history(20_000)
                .revert(),
            Code::default()
                .charge_history(0)
                .sstore(1)
                .refill_history(0)
                .revert(),
            (0, SSTORE_SET as i64, 30_000, 27_920),
        ),
    ];

    for (child, uncharged_child, reverting) in rows {
        let charged = call(Code::default().call(CHILD).op(STOP), child);
        let uncharged = call(Code::default().call(CHILD).op(STOP), uncharged_child);

        let returned = charged.child();
        assert_eq!(returned.result, InstructionResult::Revert);
        assert_eq!(counters(&returned.gas), reverting);
        assert_eq!(unwound_reservoir(&returned.gas), RESERVOIR as i64);

        // The rollback ran: the caller holds the reservoir it lent and nothing the child charged.
        assert_eq!(counters(&charged.settled), (RESERVOIR, 0, 0, 0));
        assert_eq!(charged.settled.remaining(), uncharged.settled.remaining());
        assert_eq!(charged.gas().reservoir_remaining(), RESERVOIR);
        assert_eq!(charged.total(), uncharged.total());
    }
}

/// A halting frame gives the reservoir back and burns all its regular gas, including what its
/// charges spilled: it costs its caller the gas forwarded to it and nothing more.
#[test]
fn test_a_halting_frame_returns_the_reservoir_and_burns_its_regular_gas() {
    let charged = call(
        Code::default().call(CHILD).op(STOP),
        Code::default()
            .sstore(1)
            .charge_history(HISTORY)
            .op(INVALID),
    );
    let halts_at_once = call(
        Code::default().call(CHILD).op(STOP),
        Code::default().op(INVALID),
    );

    let returned = charged.child();
    assert_eq!(returned.result, InstructionResult::InvalidFEOpcode);
    assert_eq!(
        counters(&returned.gas),
        (0, SSTORE_SET as i64, HISTORY as i64, 47_920)
    );
    assert_eq!(unwound_reservoir(&returned.gas), RESERVOIR as i64);

    assert_eq!(counters(&charged.settled), (RESERVOIR, 0, 0, 0));
    assert_eq!(
        charged.settled.remaining(),
        halts_at_once.settled.remaining()
    );
    assert_eq!(charged.total(), halts_at_once.total());
}

/// The first frame settles into the transaction the same way. When a created contract's initcode
/// charges history gas and halts, the transaction refills the create's upfront state charge and
/// then burns its regular gas again, spill included: it keeps its whole reservoir and spends all
/// of its regular gas, whatever the history charge took.
#[test]
fn test_a_halting_first_frame_returns_the_reservoir_and_burns_its_regular_gas() {
    // (reservoir, history charged, the first frame's counters as it halts)
    let rows = [
        // The create's 183,600 state gas spills 83,600 onto regular gas, so the frame inherits
        // no reservoir and the whole history charge spills.
        (RESERVOIR, HISTORY, (0, 0, HISTORY as i64, HISTORY)),
        // The create's state gas leaves 816,400 of the reservoir. The history charge takes them
        // and spills 83,600.
        (1_000_000, 900_000, (0, 0, 900_000, 83_600)),
    ];

    for (reservoir, history, halting) in rows {
        let create = |amount| {
            run(
                amsterdam(db(Code::default(), Code::default()), 0),
                tx(
                    TxKind::Create,
                    Code::default().charge_history(amount).op(INVALID).0,
                    REGULAR_CAP + reservoir,
                ),
            )
        };
        let charged = create(history);
        let uncharged = create(0);

        let [returned] = charged.returned.as_slice() else {
            panic!("only the create frame ran: {:?}", charged.returned);
        };
        assert_eq!(returned.result, InstructionResult::InvalidFEOpcode);
        assert_eq!(counters(&returned.gas), halting);
        assert_eq!(
            unwound_reservoir(&returned.gas),
            reservoir.saturating_sub(NEW_ACCOUNT) as i64,
            "the reservoir the frame inherited"
        );

        assert_eq!(counters(&charged.settled), (reservoir, 0, 0, 0));
        assert_eq!(charged.settled.remaining(), 0, "the spills are burned too");
        assert!(
            matches!(
                charged.result,
                ExecutionResult::Halt {
                    reason: HaltReason::InvalidFEOpcode,
                    ..
                }
            ),
            "{:?}",
            charged.result
        );
        // Everything but the reservoir is spent.
        assert_eq!(charged.total(), REGULAR_CAP);
        assert_eq!(charged.gas().reservoir_remaining(), reservoir);
        assert_eq!(charged.gas(), uncharged.gas());
        assert!(charged
            .state
            .get(&BENCH_CALLER.create(0))
            .is_none_or(|account| {
                !account.is_created() && account.info.code_hash == KECCAK_EMPTY
            }));
    }
}

/// With a schedule that prices history bytes, deposited code pays `len × price` as history gas
/// next to its state gas, and the state column does not see it.
#[test]
fn test_deposited_code_is_charged_history_gas_beside_its_state_gas() {
    let create = |history_per_byte| {
        run(
            amsterdam(db(Code::default(), Code::default()), history_per_byte),
            tx(TxKind::Create, initcode(), REGULAR_CAP + 1_000_000),
        )
    };
    let priced = create(HISTORY_PER_BYTE);
    let unpriced = create(0);

    let frame = priced.returned[0];
    assert_eq!(frame.result, InstructionResult::Return);
    assert_eq!(frame.gas.state_gas_spent(), CODE_STATE as i64);
    assert_eq!(frame.gas.history_gas_spent(), HISTORY_FOR_CODE);
    assert_eq!(priced.settled.history_gas_spent(), HISTORY_FOR_CODE);
    assert_eq!(unpriced.settled.history_gas_spent(), 0);

    assert_eq!(
        priced.gas().state_gas_spent_final(),
        NEW_ACCOUNT + CODE_STATE
    );
    assert_eq!(
        unpriced.gas().state_gas_spent_final(),
        NEW_ACCOUNT + CODE_STATE
    );
    assert_eq!(priced.total(), unpriced.total() + HISTORY_FOR_CODE as u64);

    let ExecutionResult::Success {
        output: Output::Create(code, Some(created)),
        ..
    } = &priced.result
    else {
        panic!("create succeeds: {:?}", priced.result);
    };
    assert_eq!(code.len() as u64, CODE_LEN);
    assert_ne!(priced.state[created].info.code_hash, KECCAK_EMPTY);
}

/// A contract created by a contract pays the same history gas, and it reaches the transaction
/// through the success merge of the creating frame.
#[test]
fn test_a_nested_deposit_merges_its_history_gas_into_the_caller() {
    let priced = run(
        amsterdam(
            db(Code::default().create_and_stop(), Code::default()),
            HISTORY_PER_BYTE,
        ),
        tx(TxKind::Call(CONTRACT), Vec::new(), REGULAR_CAP + 1_000_000),
    );

    let child = priced.child();
    assert_eq!(child.result, InstructionResult::Return);
    assert_eq!(child.gas.history_gas_spent(), HISTORY_FOR_CODE);
    assert_eq!(priced.settled.history_gas_spent(), HISTORY_FOR_CODE);
    assert_eq!(
        priced.gas().state_gas_spent_final(),
        NEW_ACCOUNT + CODE_STATE
    );
}

/// A deposit whose state gas fits but whose history gas does not fails out of gas: the created
/// account keeps no code, and the state gas already charged for the code is given back.
#[test]
fn test_a_deposit_that_cannot_pay_its_history_gas_runs_out_of_gas() {
    let create = |history_per_byte, gas_limit| {
        run(
            amsterdam(db(Code::default(), Code::default()), history_per_byte),
            tx(TxKind::Create, initcode(), gas_limit),
        )
    };
    let history = CODE_LEN * HISTORY_PER_BYTE;
    // Below the cap there is no reservoir, so every charge comes out of regular gas and a
    // transaction with exactly the unpriced cost ends with nothing left.
    let exact = create(0, REGULAR_CAP).total();
    assert!(exact + history < REGULAR_CAP);

    let paid = create(HISTORY_PER_BYTE, exact + history);
    assert!(paid.result.is_success(), "{:?}", paid.result);
    assert_eq!(paid.total(), exact + history);
    assert_eq!(paid.settled.remaining(), 0);
    assert_eq!(paid.settled.history_gas_spent(), HISTORY_FOR_CODE);

    // The same gas limit is enough when the schedule does not price the bytes.
    assert!(create(0, exact + history - 1).result.is_success());

    let short = create(HISTORY_PER_BYTE, exact + history - 1);
    assert_eq!(
        short.result,
        ExecutionResult::Halt {
            reason: HaltReason::OutOfGas(OutOfGasError::Basic),
            gas: ResultGas::new_with_state_gas(exact + history - 1, 0, short.gas().floor_gas(), 0),
            logs: Vec::new(),
        }
    );
    let frame = short.returned[0];
    assert_eq!(frame.result, InstructionResult::OutOfGas);
    assert_eq!(
        frame.gas.state_gas_spent(),
        CODE_STATE as i64,
        "the state charge went through"
    );
    assert_eq!(
        frame.gas.history_gas_spent(),
        0,
        "the history charge was refused whole"
    );
    assert_eq!(counters(&short.settled), (0, 0, 0, 0));

    // The create checkpoint was reverted: the account was not created, let alone given code.
    let created = BENCH_CALLER.create(0);
    assert!(
        short.state.get(&created).is_none_or(|account| {
            !account.is_created()
                && account.info.nonce == 0
                && account.info.code_hash == KECCAK_EMPTY
        }),
        "{:?}",
        short.state.get(&created)
    );
}

/// The deposit charge is made only where EIP-8037 is enabled: on Osaka, a schedule that prices
/// history bytes charges deposited code nothing, while on Amsterdam the same price is charged.
#[test]
fn test_a_deposit_is_not_charged_history_gas_without_eip8037() {
    let create = |spec: SpecId, history_per_byte, gas_limit| {
        let ctx = context(db(Code::default(), Code::default()), spec, history_per_byte);
        assert_eq!(
            ctx.cfg.is_amsterdam_eip8037_enabled(),
            spec == SpecId::AMSTERDAM
        );
        assert_eq!(
            ctx.cfg
                .gas_params
                .code_deposit_history_gas(CODE_LEN as usize),
            CODE_LEN * history_per_byte
        );
        run(ctx, tx(TxKind::Create, initcode(), gas_limit))
    };
    // Without EIP-8037 there is no reservoir, so the gas limit stays within the cap.
    let priced = create(SpecId::OSAKA, HISTORY_PER_BYTE, REGULAR_CAP);
    let unpriced = create(SpecId::OSAKA, 0, REGULAR_CAP);

    let [frame] = priced.returned.as_slice() else {
        panic!("only the create frame ran: {:?}", priced.returned);
    };
    assert_eq!(frame.result, InstructionResult::Return);
    assert_eq!(frame.gas.history_gas_spent(), 0);
    assert_eq!(priced.settled.history_gas_spent(), 0);
    assert_eq!(priced.settled, unpriced.settled);
    assert_eq!(priced.gas(), unpriced.gas());

    let ExecutionResult::Success {
        output: Output::Create(code, Some(created)),
        ..
    } = &priced.result
    else {
        panic!("create succeeds: {:?}", priced.result);
    };
    assert_eq!(code.len() as u64, CODE_LEN);
    assert_ne!(priced.state[created].info.code_hash, KECCAK_EMPTY);

    // The price is live: the same schedule on Amsterdam charges it.
    let amsterdam = create(SpecId::AMSTERDAM, HISTORY_PER_BYTE, REGULAR_CAP + 1_000_000);
    assert_eq!(amsterdam.settled.history_gas_spent(), HISTORY_FOR_CODE);
}

/// A frame that takes back some of its own history charge gets the spilled regular gas back first
/// and the rest as reservoir, and its counter falls by what it took back.
#[test]
fn test_a_refill_returns_the_spill_first_and_the_rest_to_the_reservoir() {
    let run = |charge, refill| {
        call(
            Code::default()
                .charge_history(charge)
                .refill_history(refill)
                .op(STOP),
            Code::default(),
        )
    };
    let baseline = run(0, 0);
    // 150,000 against a 100,000 reservoir spills 50,000; taking back 60,000 returns those 50,000
    // to regular gas and 10,000 to the reservoir.
    let refilled = run(LARGE_HISTORY, 60_000);

    assert_eq!(counters(&refilled.settled), (10_000, 0, 90_000, 0));
    assert_eq!(refilled.settled.remaining(), baseline.settled.remaining());
    assert_eq!(refilled.total(), baseline.total() + 90_000);
}

/// A child that takes back history gas its caller charged goes below zero on its own, and nets the
/// caller's charge out when it succeeds.
#[test]
fn test_a_child_refill_of_its_callers_charge_nets_out_on_success() {
    let run = |amount| {
        call(
            Code::default().charge_history(amount).call(CHILD).op(STOP),
            Code::default().refill_history(amount).op(STOP),
        )
    };
    let baseline = run(0);
    let refilled = run(LARGE_HISTORY);

    let child = refilled.child();
    assert_eq!(child.result, InstructionResult::Stop);
    // The child inherited an empty reservoir and has no spill of its own, so the whole refill
    // lands in the reservoir.
    assert_eq!(
        counters(&child.gas),
        (LARGE_HISTORY, 0, -(LARGE_HISTORY as i64), 0)
    );

    // The caller absorbs the child's reservoir last in, first out: 50,000 pay its own spill back
    // to regular gas and the other 100,000 stay reservoir. The child's net has cancelled the
    // caller's charge.
    let caller = refilled.returned[1];
    assert_eq!(caller.result, InstructionResult::Stop);
    assert_eq!(counters(&caller.gas), (RESERVOIR, 0, 0, 0));

    // The transaction ends as if nothing had been charged.
    assert_eq!(counters(&refilled.settled), (RESERVOIR, 0, 0, 0));
    assert_eq!(refilled.settled.remaining(), baseline.settled.remaining());
    assert_eq!(refilled.total(), baseline.total());
}

/// A sibling's refill pays back what an earlier sibling's history charge spilled. The caller took
/// that spill over when the first sibling succeeded, but the second sibling starts with no spill
/// of its own, so its refill lands in its reservoir. When it returns, the caller moves that
/// reservoir back to regular gas up to its outstanding spill and keeps the rest as reservoir. A
/// refill smaller than the spill leaves the rest outstanding, and the caller's regular gas short
/// by it.
#[test]
fn test_a_sibling_refill_pays_back_the_spill_of_an_earlier_siblings_history_charge() {
    // (the two siblings, the same two charging nothing,
    //  the siblings' counters and the caller's counters as each returns)
    let rows = [
        // The history charge spills 50,000; the second sibling takes the whole charge back.
        (
            [
                Code::default().charge_history(LARGE_HISTORY).op(STOP),
                Code::default().refill_history(LARGE_HISTORY).op(STOP),
            ],
            [
                Code::default().charge_history(0).op(STOP),
                Code::default().refill_history(0).op(STOP),
            ],
            [
                (0, 0, LARGE_HISTORY as i64, 50_000),
                (LARGE_HISTORY, 0, -(LARGE_HISTORY as i64), 0),
            ],
            (RESERVOIR, 0, 0, 0),
        ),
        // The write leaves 2,080 of the reservoir, and the history charge takes them and spills
        // 47,920. The second sibling clears the slot in the caller's storage and refills 97,920
        // state gas: 47,920 pay back the history charge's spill, 50,000 stay reservoir.
        (
            [
                Code::default().sstore(1).charge_history(HISTORY).op(STOP),
                Code::default().sstore(0).op(STOP),
            ],
            [
                Code::default().sstore(1).charge_history(0).op(STOP),
                Code::default().sstore(0).op(STOP),
            ],
            [
                (0, SSTORE_SET as i64, HISTORY as i64, 47_920),
                (SSTORE_SET, -(SSTORE_SET as i64), 0, 0),
            ],
            (RESERVOIR - HISTORY, 0, HISTORY as i64, 0),
        ),
        // The second sibling takes back only 20,000 of the charge, and returns them as reservoir.
        // The caller pays all 20,000 back to regular gas, so 30,000 of its 50,000 spill stay
        // outstanding, and its history net is 150,000 − 20,000 = 130,000.
        (
            [
                Code::default().charge_history(LARGE_HISTORY).op(STOP),
                Code::default().refill_history(20_000).op(STOP),
            ],
            [
                Code::default().charge_history(0).op(STOP),
                Code::default().refill_history(0).op(STOP),
            ],
            [
                (0, 0, LARGE_HISTORY as i64, 50_000),
                (20_000, 0, -20_000, 0),
            ],
            (0, 0, 130_000, 30_000),
        ),
        // The second sibling first charges 20,000 history of its own. It inherited an empty
        // reservoir, so all of it spills. Taking back the 150,000 then pays those 20,000 back and
        // leaves 130,000 as reservoir and a net of −130,000. The caller's history net is
        // 150,000 − 130,000 = 20,000; 50,000 of the reservoir pay back its spill and 80,000 stay.
        (
            [
                Code::default().charge_history(LARGE_HISTORY).op(STOP),
                Code::default()
                    .charge_history(20_000)
                    .refill_history(LARGE_HISTORY)
                    .op(STOP),
            ],
            [
                Code::default().charge_history(0).op(STOP),
                Code::default().charge_history(0).refill_history(0).op(STOP),
            ],
            [
                (0, 0, LARGE_HISTORY as i64, 50_000),
                (130_000, 0, -130_000, 0),
            ],
            (80_000, 0, 20_000, 0),
        ),
    ];

    for (siblings, uncharged_siblings, sibling_gas, caller_gas) in rows {
        let charged = call_siblings(siblings);
        let uncharged = call_siblings(uncharged_siblings);

        let [first, second, caller] = charged.returned.as_slice() else {
            panic!("two siblings and the caller: {:?}", charged.returned);
        };
        for (sibling, gas) in [first, second].into_iter().zip(sibling_gas) {
            assert_eq!(sibling.result, InstructionResult::Stop);
            assert_eq!(counters(&sibling.gas), gas);
        }

        assert_eq!(caller.result, InstructionResult::Stop);
        assert_eq!(counters(&caller.gas), caller_gas);
        assert_eq!(unwound_reservoir(&caller.gas), RESERVOIR as i64);

        // The caller's regular gas is short of the uncharged run's by the spill still outstanding,
        // and the history net still charged is the only difference in the total.
        let (reservoir, _, history, spilled) = caller_gas;
        assert_eq!(counters(&charged.settled), caller_gas);
        assert_eq!(
            charged.settled.remaining(),
            uncharged.settled.remaining() - spilled
        );
        assert_eq!(unwound_reservoir(&charged.settled), RESERVOIR as i64);
        assert_eq!(charged.gas().reservoir_remaining(), reservoir);
        assert_eq!(charged.gas().state_gas_spent_final(), 0);
        assert_eq!(
            charged.total(),
            uncharged.total() + u64::try_from(history).unwrap()
        );
    }
}

/// A sibling that takes back an earlier sibling's history charge and then fails takes the refill
/// back out as it unwinds, so the caller absorbs no reservoir: the spill it took over from the
/// first sibling stays outstanding, and its regular gas stays short by it.
#[test]
fn test_a_failed_sibling_refill_leaves_the_spill_of_an_earlier_siblings_history_charge() {
    // (the second sibling, the same sibling taking nothing back, the result it returns with)
    let rows = [
        (
            Code::default().refill_history(LARGE_HISTORY).revert(),
            Code::default().refill_history(0).revert(),
            InstructionResult::Revert,
        ),
        (
            Code::default().refill_history(LARGE_HISTORY).op(INVALID),
            Code::default().refill_history(0).op(INVALID),
            InstructionResult::InvalidFEOpcode,
        ),
    ];

    for (second_sibling, uncharged_second_sibling, result) in rows {
        let charged = call_siblings([
            Code::default().charge_history(LARGE_HISTORY).op(STOP),
            second_sibling,
        ]);
        let uncharged = call_siblings([
            Code::default().charge_history(0).op(STOP),
            uncharged_second_sibling,
        ]);

        let [first, second, caller] = charged.returned.as_slice() else {
            panic!("two siblings and the caller: {:?}", charged.returned);
        };
        // The history charge spills 50,000.
        assert_eq!(first.result, InstructionResult::Stop);
        assert_eq!(counters(&first.gas), (0, 0, LARGE_HISTORY as i64, 50_000));
        // The second sibling inherited an empty reservoir and has no spill of its own, so the
        // whole refill lands in its reservoir, and unwinding takes all 150,000 back out.
        assert_eq!(second.result, result);
        assert_eq!(
            counters(&second.gas),
            (LARGE_HISTORY, 0, -(LARGE_HISTORY as i64), 0)
        );
        assert_eq!(
            unwound_reservoir(&second.gas),
            0,
            "the reservoir it inherited"
        );

        // The caller absorbs the empty reservoir the second sibling unwound to, so nothing pays
        // back its 50,000 spill.
        let outstanding = (0, 0, LARGE_HISTORY as i64, 50_000);
        assert_eq!(caller.result, InstructionResult::Stop);
        assert_eq!(counters(&caller.gas), outstanding);
        assert_eq!(unwound_reservoir(&caller.gas), RESERVOIR as i64);

        assert_eq!(counters(&charged.settled), outstanding);
        assert_eq!(
            charged.settled.remaining(),
            uncharged.settled.remaining() - 50_000
        );
        assert_eq!(unwound_reservoir(&charged.settled), RESERVOIR as i64);
        assert_eq!(charged.gas().reservoir_remaining(), 0);
        assert_eq!(charged.gas().state_gas_spent_final(), 0);
        assert_eq!(charged.total(), uncharged.total() + LARGE_HISTORY);
    }
}

/// A child that takes back its caller's history charge and then reverts takes the refill back
/// too: the caller's charge stands.
#[test]
fn test_a_reverted_child_undoes_its_refill_of_its_callers_charge() {
    let run = |amount| {
        call(
            Code::default().charge_history(amount).call(CHILD).op(STOP),
            Code::default().refill_history(amount).revert(),
        )
    };
    let baseline = run(0);
    let refilled = run(LARGE_HISTORY);

    let child = refilled.child();
    assert_eq!(child.result, InstructionResult::Revert);
    assert_eq!(
        counters(&child.gas),
        (LARGE_HISTORY, 0, -(LARGE_HISTORY as i64), 0)
    );
    assert_eq!(
        unwound_reservoir(&child.gas),
        0,
        "the reservoir it inherited"
    );

    assert_eq!(
        counters(&refilled.settled),
        (0, 0, LARGE_HISTORY as i64, 50_000)
    );
    assert_eq!(refilled.total(), baseline.total() + LARGE_HISTORY);
}

/// History gas a successful child charged reaches the transaction's counter but not the state gas
/// the result reports; it is part of the total, so upstream's derived regular column carries it.
#[test]
fn test_history_gas_stays_out_of_the_state_gas_column() {
    let run = |amount| {
        call(
            Code::default().call(CHILD).op(STOP),
            Code::default().sstore(1).charge_history(amount).op(STOP),
        )
    };
    let charged = run(HISTORY);
    let uncharged = run(0);

    assert_eq!(charged.child().gas.history_gas_spent(), HISTORY as i64);
    assert_eq!(charged.settled.history_gas_spent(), HISTORY as i64);
    assert_eq!(charged.settled.state_gas_spent(), SSTORE_SET as i64);

    assert_eq!(charged.gas().state_gas_spent_final(), SSTORE_SET);
    assert_eq!(charged.gas().block_state_gas_used(), SSTORE_SET);
    assert_eq!(uncharged.gas().state_gas_spent_final(), SSTORE_SET);
    assert_eq!(charged.total(), uncharged.total() + HISTORY);
    assert_eq!(
        charged.gas().block_regular_gas_used(),
        uncharged.gas().block_regular_gas_used() + HISTORY
    );
}

/// A block-level consumer takes history gas out of the regular column before the calldata floor
/// applies: `max(total − state − history, floor)`. `block_regular_gas_used()` has applied the
/// floor already, so taking history out of it afterwards can land below the floor.
#[test]
fn test_history_gas_comes_out_of_the_regular_column_before_the_floor() {
    // The call's intrinsic gas is 15,000 plus 16 per non-zero calldata byte, and its code costs 3
    // (one `PUSH4`), so it spends 15,003 + 16 × bytes regular gas. Its floor is 15,000 + 64 × bytes.
    // (calldata bytes, total_gas_spent, floor_gas, regular column, the subtraction after the floor)
    let rows = [
        // Regular gas 23,003 is below the 47,000 floor, and the total with history is above it.
        (500, 73_003, 47_000, 47_000, 23_003),
        // Regular gas 34,203 and the total with history are both below the 91,800 floor.
        (1_200, 84_203, 91_800, 91_800, 41_800),
    ];

    for (bytes, total, floor, regular, after_floor) in rows {
        let send = |amount| {
            run(
                amsterdam(
                    db(
                        Code::default().charge_history(amount).op(STOP),
                        Code::default(),
                    ),
                    0,
                ),
                tx(
                    TxKind::Call(CONTRACT),
                    vec![0xff; bytes],
                    REGULAR_CAP + RESERVOIR,
                ),
            )
        };
        let charged = send(HISTORY);
        let uncharged = send(0);

        // The whole charge came out of the reservoir.
        assert_eq!(
            counters(&charged.settled),
            (RESERVOIR - HISTORY, 0, HISTORY as i64, 0)
        );
        let history = u64::try_from(charged.settled.history_gas_spent()).unwrap();
        let gas = charged.gas();
        assert_eq!(
            (
                gas.total_gas_spent(),
                gas.state_gas_spent_final(),
                gas.floor_gas()
            ),
            (total, 0, floor),
            "{bytes} calldata bytes"
        );

        let column = gas
            .total_gas_spent()
            .saturating_sub(gas.state_gas_spent_final())
            .saturating_sub(history)
            .max(gas.floor_gas());
        assert_eq!(column, regular);
        assert_eq!(column, uncharged.gas().block_regular_gas_used());

        // The trap: the floor is already in `block_regular_gas_used()`.
        assert_eq!(gas.block_regular_gas_used(), total.max(floor));
        assert_eq!(gas.block_regular_gas_used() - history, after_floor);
        assert!(after_floor < floor);
    }
}
