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

/// An Amsterdam context whose schedule charges `history_per_byte` for deposited code.
fn amsterdam(db: Db, history_per_byte: u64) -> TestContext {
    Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
        cfg.tx_gas_limit_cap = Some(REGULAR_CAP);
        cfg.gas_params
            .override_gas([(GasId::code_deposit_history_gas(), history_per_byte)]);
    })
}

fn run(ctx: TestContext, tx: TxEnv) -> Run {
    let mut instructions = EthInstructions::new_mainnet_with_spec(SpecId::AMSTERDAM);
    instructions.insert_instruction(CHARGE_HISTORY, Instruction::new(charge_history), 0);
    instructions.insert_instruction(REFILL_HISTORY, Instruction::new(refill_history), 0);
    instructions.insert_instruction(CHARGE_STATE, Instruction::new(charge_state), 0);
    let mut evm: TestEvm = Evm::new(ctx, instructions, EthPrecompiles::new(SpecId::AMSTERDAM));
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
        let code = code.build();
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    db
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
    run(
        amsterdam(db(contract, child), 0),
        tx(TxKind::Call(CONTRACT), Vec::new(), REGULAR_CAP + RESERVOIR),
    )
}

/// `reservoir + state_spent + history_spent − spilled`: what a rollback sets the reservoir to,
/// which must be the reservoir the frame inherited.
const fn unwound_reservoir(gas: &GasTracker) -> i64 {
    gas.reservoir() as i64 + gas.state_gas_spent() + gas.history_gas_net()
        - gas.state_gas_spilled() as i64
}

/// `(reservoir, state_spent, history_net, spilled)`.
const fn counters(gas: &GasTracker) -> (u64, i64, i64, u64) {
    (
        gas.reservoir(),
        gas.state_gas_spent(),
        gas.history_gas_net(),
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
        assert_eq!(history.settled.history_gas_spent(), amount);
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
    assert_eq!(frame.gas.history_gas_spent(), CODE_LEN * HISTORY_PER_BYTE);
    assert_eq!(
        priced.settled.history_gas_spent(),
        CODE_LEN * HISTORY_PER_BYTE
    );
    assert_eq!(unpriced.settled.history_gas_spent(), 0);

    assert_eq!(
        priced.gas().state_gas_spent_final(),
        NEW_ACCOUNT + CODE_STATE
    );
    assert_eq!(
        unpriced.gas().state_gas_spent_final(),
        NEW_ACCOUNT + CODE_STATE
    );
    assert_eq!(
        priced.total(),
        unpriced.total() + CODE_LEN * HISTORY_PER_BYTE
    );

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
    assert_eq!(child.gas.history_gas_spent(), CODE_LEN * HISTORY_PER_BYTE);
    assert_eq!(
        priced.settled.history_gas_spent(),
        CODE_LEN * HISTORY_PER_BYTE
    );
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
    assert_eq!(paid.settled.history_gas_spent(), history);

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
        frame.gas.history_gas_net(),
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
    assert_eq!(child.gas.history_gas_spent(), 0, "the unsigned view clamps");

    // The caller's 50,000 spill stays spent as regular gas and comes back as reservoir.
    assert_eq!(counters(&refilled.settled), (LARGE_HISTORY, 0, 0, 50_000));
    assert_eq!(refilled.total(), baseline.total());
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

    assert_eq!(charged.child().gas.history_gas_spent(), HISTORY);
    assert_eq!(charged.settled.history_gas_spent(), HISTORY);
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
