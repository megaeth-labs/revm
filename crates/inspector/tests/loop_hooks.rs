//! The two points the handler loops give a consumer's EVM, on the plain and the inspected path.
//!
//! A pending action: a consumer sets a frame's next action on its interpreter before the frame
//! runs, at its start or when a child returned into it, and the frame's run returns that action
//! before any instruction runs, without an inspector `step`. A return ends the frame; a new frame
//! starts a child, and the frame resumes on its result. A frame whose action is set at its start
//! is not handed to the inspector's `initialize_interp`, so the inspector cannot replace it.
//!
//! A frame's run result: the result of a frame that ran reaches the consumer's
//! [`EvmTr::frame_run_result`] once, as it leaves the frame's run, before the inspector's
//! `call_end` and `frame_end` and before its caller; what the consumer leaves in it is what they
//! see. A result given without a run (a precompile's, an inspector's answer) does not reach it.
//!
//! The consumer here wraps revm's EVM and changes nothing else: [`PARENT`] calls [`CHILD`], then
//! marks a slot of its own, and returns the call's status and return data; [`CHILD`] marks a slot
//! of its own and halts. A transaction goes to [`PARENT`] unless a test sends it to [`CHILD`]. A
//! slot left unmarked is an instruction that did not run. The consumer's hook writes its line into
//! the inspector's, so one list orders both.

use context::{
    result::{EVMError, ExecutionResult, Output},
    Cfg, Context, ContextSetters, ContextTr, Evm, FrameStack, JournalTr, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    evm::{ContextDbError, FrameInitResult},
    instructions::EthInstructions,
    EthFrame, EthPrecompiles, EvmTr, FrameInitOrResult, FrameResult, Handler, ItemOrResult,
    MainBuilder, MainContext, MainnetContext, MainnetHandler,
};
use interpreter::{
    interpreter::{EthInterpreter, ExtBytecode},
    interpreter_action::FrameInit,
    interpreter_types::{Jumps, LoopControl},
    CallInputs, CallOutcome, CreateInputs, CreateScheme, FrameInput, Gas, InputsImpl,
    InstructionResult, Interpreter, InterpreterAction, InterpreterResult, SharedMemory,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_inspector::{inspect_instructions, Inspector, InspectorEvmTr, InspectorHandler};
use state::{
    bytecode::opcode::{
        ADD, CALL, INVALID, MSTORE, PUSH0, PUSH1, PUSH20, PUSH3, RETURN, RETURNDATACOPY,
        RETURNDATASIZE, SSTORE,
    },
    AccountInfo, Bytecode,
};
use std::convert::Infallible;

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;
type Inner = Evm<Ctx, Recorder, EthInstructions<EthInterpreter, Ctx>, EthPrecompiles, EthFrame>;
type Error = EVMError<Infallible>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 1_000_000;

/// Calls [`CHILD`] (or the [`Callee`] a test names) with [`CHILD_GAS`], marks its slot 0, and
/// returns the call's status as a word followed by the call's return data.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Marks its slot 0 and halts on `INVALID`.
const CHILD: Address = address!("0x00000000000000000000000000000000000c0de1");
/// The gas [`PARENT`] forwards to [`CHILD`], whose halt spends it: 0x0186a0.
const CHILD_GAS: [u8; 3] = [0x01, 0x86, 0xa0];
/// The identity precompile, which revm answers without running a frame.
const IDENTITY: Address = address!("0x0000000000000000000000000000000000000004");
/// The gas the consumer takes from [`CHILD`] for the creation it starts there.
const CREATION_GAS: u64 = 50_000;

/// The output of the action the consumer sets on [`CHILD`] at its start.
const AT_START: &[u8] = b"set at its start";
/// The output of the action the consumer sets on [`PARENT`] when [`CHILD`] returns into it.
const ON_RESUME: &[u8] = b"set on its resume";
/// The output of the action the consumer sets on [`CHILD`] when the creation it started there
/// returns into it.
const AFTER_ITS_CHILD: &[u8] = b"set once its child returned";
/// The output of the revert the consumer's hook makes of [`CHILD`]'s halt.
const REWRITTEN: &[u8] = b"rewritten as it leaves its run";
/// The output of the revert the inspector answers the call to [`CHILD`] with.
const ANSWERED: &[u8] = b"answered by the inspector";
/// The output of the return the inspector sets on [`CHILD`]'s frame in `initialize_interp`.
const AT_INIT: &[u8] = b"set in initialize_interp";
/// The output the inspector's `step` writes into a return pending on the frame it steps.
const IN_STEP: &[u8] = b"edited in step";

/// What the consumer does at the loops' points.
#[derive(Clone, Copy, Debug, Default)]
struct Plan {
    /// What the consumer sets on [`CHILD`]'s frame, once revm built it.
    at_start: Option<AtStart>,
    /// A return carrying [`ON_RESUME`] on [`PARENT`]'s frame, once [`CHILD`]'s result is in it.
    on_resume: bool,
    /// [`CHILD`]'s halt made a revert carrying [`REWRITTEN`] as it leaves its run.
    rewrite: bool,
}

/// The action the consumer sets on [`CHILD`]'s frame before it runs.
#[derive(Clone, Copy, Debug)]
enum AtStart {
    /// A return carrying [`AT_START`].
    Return,
    /// A creation with no init code on [`CREATION_GAS`] and the shares of the frame's reservoir
    /// and pools; once it returned into [`CHILD`], a return carrying [`AFTER_ITS_CHILD`].
    Create,
    /// The creation of [`AtStart::Create`]; once it returned into [`CHILD`], nothing, so
    /// [`CHILD`] runs its code.
    CreateThenRun,
}

/// What [`PARENT`] calls, and whether the call runs.
#[derive(Clone, Copy, Debug)]
enum Callee {
    /// [`CHILD`], which runs.
    Child,
    /// [`IDENTITY`], which revm answers.
    Precompile,
    /// [`CHILD`], which the inspector answers with a revert carrying [`ANSWERED`].
    Answered,
    /// [`CHILD`], which the inspector halts out of gas in `initialize_interp`.
    HaltedAtInit,
    /// [`CHILD`], which the inspector returns with [`AT_INIT`] in `initialize_interp`, and whose
    /// pending return its `step` edits to carry [`IN_STEP`].
    ReturnedAtInit,
}

/// What the inspector saw, one line per callback, by the account the frame runs as.
#[derive(Debug, Default)]
struct Recorder {
    events: Vec<String>,
    /// Answers the call to [`CHILD`].
    answer: bool,
    /// Halts [`CHILD`]'s frame out of gas in `initialize_interp`.
    halt_at_init: bool,
    /// Returns [`CHILD`]'s frame with [`AT_INIT`] in `initialize_interp`, and edits a return
    /// pending on a frame it steps to carry [`IN_STEP`].
    return_at_init: bool,
}

const fn name(address: Address) -> &'static str {
    match address {
        PARENT => "PARENT",
        CHILD => "CHILD",
        IDENTITY => "IDENTITY",
        _ => "other",
    }
}

fn target(input: &FrameInput) -> &'static str {
    match input {
        FrameInput::Call(inputs) => name(inputs.target_address),
        FrameInput::Create(_) => "CREATION",
        FrameInput::Empty => "other",
    }
}

fn is_call_to(input: &FrameInput, address: Address) -> bool {
    matches!(input, FrameInput::Call(inputs) if inputs.target_address == address)
}

/// A return of `frame` carrying `output`, on the gas it holds.
const fn returning(frame: &EthFrame, output: &'static [u8]) -> InterpreterAction {
    InterpreterAction::new_return(
        InstructionResult::Return,
        Bytes::from_static(output),
        frame.interpreter.gas,
    )
}

impl Inspector<Ctx> for Recorder {
    fn initialize_interp(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        let account = name(interp.input.target_address);
        self.events.push(format!("initialize_interp {account}"));
        if self.halt_at_init && interp.input.target_address == CHILD {
            interp.halt(InstructionResult::OutOfGas);
        }
        if self.return_at_init && interp.input.target_address == CHILD {
            interp.return_with_output(Bytes::from_static(AT_INIT));
        }
    }

    fn step(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        let (account, pc) = (name(interp.input.target_address), interp.bytecode.pc());
        self.events.push(format!("step {account} {pc}"));
        if self.return_at_init {
            if let Some(InterpreterAction::Return(result)) = interp.bytecode.action() {
                result.output = Bytes::from_static(IN_STEP);
            }
        }
    }

    fn step_end(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        let (account, pc) = (name(interp.input.target_address), interp.bytecode.pc());
        self.events.push(format!("step_end {account} {pc}"));
    }

    fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        if !self.answer || inputs.target_address != CHILD {
            return None;
        }
        let result = InterpreterResult::new(
            InstructionResult::Revert,
            Bytes::from_static(ANSWERED),
            Gas::new(inputs.gas_limit),
        );
        Some(CallOutcome::new(
            result,
            inputs.return_memory_offset.clone(),
        ))
    }

    fn call_end(&mut self, _: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let (result, output) = (outcome.result.result, &outcome.result.output);
        let account = name(inputs.target_address);
        self.events
            .push(format!("call_end {account} {result:?} {output}"));
    }

    fn frame_end(&mut self, _: &mut Ctx, input: &FrameInput, result: &mut FrameResult) {
        let (account, result) = (target(input), result.instruction_result());
        self.events.push(format!("frame_end {account} {result:?}"));
    }
}

/// revm's EVM, with a consumer's actions set as [`Plan`] says.
struct Consumer {
    evm: Inner,
    plan: Plan,
}

impl EvmTr for Consumer {
    type Context = Ctx;
    type Instructions = EthInstructions<EthInterpreter, Ctx>;
    type Precompiles = EthPrecompiles;
    type Frame = EthFrame;

    fn all(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
    ) {
        self.evm.all()
    }

    fn all_mut(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
    ) {
        self.evm.all_mut()
    }

    fn frame_init(
        &mut self,
        frame_input: FrameInit,
    ) -> Result<FrameInitResult<'_, EthFrame>, ContextDbError<Ctx>> {
        let at_start = self
            .plan
            .at_start
            .filter(|_| is_call_to(&frame_input.frame_input, CHILD));
        let params = self.evm.ctx.cfg().gas_params().clone();
        let mut started = self.evm.frame_init(frame_input)?;
        if let (Some(at_start), ItemOrResult::Item(frame)) = (at_start, &mut started) {
            let action = match at_start {
                AtStart::Return => returning(frame, AT_START),
                AtStart::Create | AtStart::CreateThenRun => {
                    let gas = &mut frame.interpreter.gas;
                    assert!(
                        gas.record_regular_cost(CREATION_GAS),
                        "the frame pays the creation"
                    );
                    let shares = gas.tracker_mut().forward_to_child(&params);
                    let mut inputs = CreateInputs::new(
                        CHILD,
                        CreateScheme::Create,
                        U256::ZERO,
                        Bytes::new(),
                        CREATION_GAS,
                        shares.reservoir,
                    );
                    inputs.set_pool_shares(shares.pools);
                    InterpreterAction::NewFrame(FrameInput::Create(Box::new(inputs)))
                }
            };
            frame.interpreter.bytecode.set_action(action);
        }
        Ok(started)
    }

    fn frame_run(&mut self) -> Result<FrameInitOrResult<EthFrame>, ContextDbError<Ctx>> {
        self.evm.frame_run()
    }

    fn frame_return_result(
        &mut self,
        result: FrameResult,
    ) -> Result<Option<FrameResult>, ContextDbError<Ctx>> {
        let returned = self.evm.frame_return_result(result)?;
        if returned.is_some() {
            return Ok(returned);
        }
        let caller = self.evm.frame_stack.get();
        let created = matches!(self.plan.at_start, Some(AtStart::Create));
        let output = if is_call_to(&caller.input, CHILD) && created {
            AFTER_ITS_CHILD
        } else if is_call_to(&caller.input, PARENT) && self.plan.on_resume {
            ON_RESUME
        } else {
            return Ok(None);
        };
        let action = returning(caller, output);
        caller.interpreter.bytecode.set_action(action);
        Ok(None)
    }

    /// Writes the result it got into the inspector's lines, by the frame on top of the stack, and
    /// rewrites [`CHILD`]'s halt as [`Plan::rewrite`] says.
    fn frame_run_result(&mut self, result: &mut FrameResult) -> Result<(), ContextDbError<Ctx>> {
        let frame = self.evm.frame_stack.get();
        let (account, got) = (target(&frame.input), result.instruction_result());
        self.evm
            .inspector
            .events
            .push(format!("hook {account} {got:?}"));
        if self.plan.rewrite && got == InstructionResult::InvalidFEOpcode {
            let result = result.interpreter_result_mut();
            result.result = InstructionResult::Revert;
            result.output = Bytes::from_static(REWRITTEN);
        }
        Ok(())
    }
}

impl InspectorEvmTr for Consumer {
    type Inspector = Recorder;

    fn all_inspector(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
        &Self::Inspector,
    ) {
        self.evm.all_inspector()
    }

    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
        &mut Self::Inspector,
    ) {
        self.evm.all_mut_inspector()
    }
}

/// [`PARENT`]'s code, calling `callee`.
fn parent_code(callee: Address) -> Bytecode {
    let mut code = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20];
    code.extend_from_slice(callee.as_slice());
    code.push(PUSH3);
    code.extend_from_slice(&CHILD_GAS);
    code.extend_from_slice(&[
        CALL,
        // Mark slot 0: the frame resumed.
        PUSH1,
        1,
        PUSH0,
        SSTORE,
        // The status at 0, the return data after it.
        PUSH0,
        MSTORE,
        RETURNDATASIZE,
        PUSH0,
        PUSH1,
        32,
        RETURNDATACOPY,
        RETURNDATASIZE,
        PUSH1,
        32,
        ADD,
        PUSH0,
        RETURN,
    ]);
    Bytecode::new_raw(code.into())
}

/// [`CHILD`]'s code.
fn child_code() -> Bytecode {
    Bytecode::new_raw(Bytes::from_static(&[PUSH1, 1, PUSH0, SSTORE, INVALID]))
}

/// What one transaction did.
#[derive(Debug)]
struct Run {
    result: ExecutionResult,
    /// The inspector's lines; empty on the plain path.
    events: Vec<String>,
    /// Whether [`PARENT`]'s and [`CHILD`]'s slot 0 hold the mark.
    marked: (bool, bool),
}

impl Run {
    fn output(&self) -> &Bytes {
        match &self.result {
            ExecutionResult::Success {
                output: Output::Call(output),
                ..
            } => output,
            other => panic!("the transaction returns, got {other:?}"),
        }
    }

    /// The lines of `account`'s steps.
    fn steps(&self, account: &str) -> Vec<&str> {
        let prefix = format!("step {account} ");
        self.lines(|line| line.starts_with(&prefix))
    }

    /// The consumer's hook lines.
    fn hooks(&self) -> Vec<&str> {
        self.lines(|line| line.starts_with("hook "))
    }

    /// Every line but the steps: the frames' starts and ends, and the hook's.
    fn ends(&self) -> Vec<&str> {
        self.lines(|line| !line.starts_with("step"))
    }

    /// The lines `keep` keeps, in order.
    fn lines(&self, keep: impl Fn(&str) -> bool) -> Vec<&str> {
        self.events
            .iter()
            .map(String::as_str)
            .filter(|line| keep(line))
            .collect()
    }
}

/// Runs a transaction to [`PARENT`], calling [`CHILD`], through the mainnet handler on the
/// consumer's EVM, inspected or not.
fn run(plan: Plan, inspected: bool) -> Run {
    run_calling(Callee::Child, plan, inspected)
}

/// [`run`], with [`PARENT`] calling `callee`.
fn run_calling(callee: Callee, plan: Plan, inspected: bool) -> Run {
    run_to(PARENT, callee, plan, inspected)
}

/// [`run_calling`], with the transaction sent to `to`.
fn run_to(to: Address, callee: Callee, plan: Plan, inspected: bool) -> Run {
    let (address, answer, halt_at_init, return_at_init) = match callee {
        Callee::Child => (CHILD, false, false, false),
        Callee::Precompile => (IDENTITY, false, false, false),
        Callee::Answered => (CHILD, true, false, false),
        Callee::HaltedAtInit => (CHILD, false, true, false),
        Callee::ReturnedAtInit => (CHILD, false, false, true),
    };
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    for (address, code) in [(PARENT, parent_code(address)), (CHILD, child_code())] {
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    let ctx = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SPEC));
    let mut consumer = Consumer {
        evm: ctx.build_mainnet_with_inspector(Recorder {
            answer,
            halt_at_init,
            return_at_init,
            ..Recorder::default()
        }),
        plan,
    };
    consumer.evm.ctx.set_tx(
        TxEnv::builder_for_bench()
            .kind(TxKind::Call(to))
            .gas_price(0)
            .gas_limit(GAS_LIMIT)
            .build_fill(),
    );

    let mut handler = MainnetHandler::<Consumer, Error, EthFrame>::default();
    let result = if inspected {
        handler.inspect_run(&mut consumer)
    } else {
        handler.run(&mut consumer)
    }
    .expect("the transaction runs");

    let state = consumer.evm.ctx.journal_mut().finalize();
    let mark = |address: Address| {
        state
            .get(&address)
            .and_then(|account| account.storage.get(&U256::ZERO))
            .is_some_and(|slot| slot.present_value == U256::from(1))
    };
    Run {
        result,
        events: core::mem::take(&mut consumer.evm.inspector.events),
        marked: (mark(PARENT), mark(CHILD)),
    }
}

/// The 32-byte status word of a call, followed by its return data.
fn status_and(success: bool, data: &[u8]) -> Bytes {
    let mut output = U256::from(success as u8).to_be_bytes_vec();
    output.extend_from_slice(data);
    output.into()
}

/// Without a plan the consumer is revm: the frames run as their code says.
#[test]
fn test_without_a_pending_action_every_frame_runs() {
    for inspected in [false, true] {
        let run = run(Plan::default(), inspected);

        assert_eq!(run.output(), &status_and(false, &[]), "{inspected}");
        assert_eq!(
            run.marked,
            (true, false),
            "the halt took the child's mark back"
        );
        if inspected {
            assert_eq!(run.steps("CHILD").len(), 4, "PUSH1 PUSH0 SSTORE INVALID");
        }
    }
}

/// An interpreter whose next action is set before it runs returns that action: no instruction
/// runs and the inspector sees no `step` or `step_end`.
#[test]
fn test_inspect_instructions_returns_a_pending_action_without_a_step() {
    let mut ctx: Ctx = Context::mainnet().with_db(Db::default());
    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(child_code()),
        InputsImpl::default(),
        false,
        SPEC,
        GAS_LIMIT,
    );
    let action = InterpreterAction::new_return(
        InstructionResult::Return,
        Bytes::from_static(AT_START),
        interpreter.gas,
    );
    interpreter.bytecode.set_action(action.clone());
    let instructions = EthInstructions::<EthInterpreter, Ctx>::new_mainnet_with_spec(SPEC);
    let mut recorder = Recorder::default();

    let returned = inspect_instructions(
        &mut ctx,
        &mut interpreter,
        &mut recorder,
        instructions.instruction_table(),
        instructions.gas_table(),
    );

    assert_eq!(returned, action);
    assert_eq!(recorder.events, Vec::<String>::new());
    assert_eq!(interpreter.bytecode.pc(), 0, "no instruction ran");
    assert_eq!(
        interpreter.gas.remaining(),
        GAS_LIMIT,
        "no instruction charged"
    );
}

/// An action the consumer sets on a frame revm built is the frame's result: its code does not
/// run, on either path, and the inspector sees the frame start and end with nothing between, no
/// `initialize_interp` and no step.
#[test]
fn test_an_action_set_at_a_frames_start_is_its_result() {
    let plan = Plan {
        at_start: Some(AtStart::Return),
        ..Plan::default()
    };
    for inspected in [false, true] {
        let run = run(plan, inspected);

        assert_eq!(run.output(), &status_and(true, AT_START), "{inspected}");
        assert_eq!(run.marked, (true, false), "the child ran no instruction");
        assert_eq!(run.hooks(), ["hook CHILD Return", "hook PARENT Return"]);
        if inspected {
            let seen = run.lines(|line| line.contains("CHILD"));
            let end = format!("call_end CHILD Return {}", Bytes::from_static(AT_START));
            assert_eq!(seen, ["hook CHILD Return", &end, "frame_end CHILD Return"]);
        }
    }
}

/// A frame whose action the consumer set at its start is not handed to `initialize_interp`, so an
/// inspector that halts the frame there does not replace the consumer's action: the consumer's
/// action is the frame's result.
#[test]
fn test_an_inspector_cannot_halt_a_frame_whose_action_is_set_at_its_start() {
    let plan = Plan {
        at_start: Some(AtStart::Return),
        ..Plan::default()
    };
    let run = run_calling(Callee::HaltedAtInit, plan, true);

    assert_eq!(run.output(), &status_and(true, AT_START));
    let end = format!("call_end CHILD Return {}", Bytes::from_static(AT_START));
    assert_eq!(
        run.lines(|line| line.contains("CHILD")),
        ["hook CHILD Return", &end, "frame_end CHILD Return"]
    );
}

/// An inspector that halts a frame in `initialize_interp` ends it before its first instruction
/// and before any `step` (upstream's run calls `step` once first): the halt is the frame's result,
/// and its caller gets a failed call and none of its gas back, the transaction's result when the
/// frame's own `INVALID` halts it.
#[test]
fn test_an_inspectors_halt_in_initialize_interp_ends_the_frame_without_a_step() {
    let halted = run_calling(Callee::HaltedAtInit, Plan::default(), true);

    assert_eq!(halted.output(), &status_and(false, &[]));
    assert_eq!(halted.result, run(Plan::default(), true).result);
    assert_eq!(
        halted.lines(|line| line.contains("CHILD")),
        [
            "initialize_interp CHILD",
            "hook CHILD OutOfGas",
            "call_end CHILD OutOfGas 0x",
            "frame_end CHILD OutOfGas"
        ]
    );
}

/// An action an inspector sets in `initialize_interp` is returned before the frame's first `step`
/// (upstream's run calls that `step` first), so whatever that `step` would do is not done: here
/// the `step` edits the pending return, and the frame returns what `initialize_interp` set, where
/// upstream's returns the edit.
#[test]
fn test_an_inspectors_action_in_initialize_interp_is_returned_without_the_step_that_edits_it() {
    let run = run_calling(Callee::ReturnedAtInit, Plan::default(), true);

    assert_eq!(run.output(), &status_and(true, AT_INIT));
    assert_eq!(run.marked, (true, false), "the child runs no instruction");
    let end = format!("call_end CHILD Return {}", Bytes::from_static(AT_INIT));
    assert_eq!(
        run.lines(|line| line.contains("CHILD")),
        [
            "initialize_interp CHILD",
            "hook CHILD Return",
            &end,
            "frame_end CHILD Return"
        ]
    );
}

/// An action the consumer sets on a caller once its child's result is in it is the caller's
/// result: the rest of its code does not run, and the inspector sees no step of it after the
/// child's end.
#[test]
fn test_an_action_set_on_a_frames_resume_is_its_result() {
    let plan = Plan {
        on_resume: true,
        ..Plan::default()
    };
    for inspected in [false, true] {
        let run = run(plan, inspected);

        assert_eq!(run.output(), &Bytes::from_static(ON_RESUME), "{inspected}");
        assert_eq!(
            run.marked,
            (false, false),
            "the parent ran nothing after the call"
        );
        if inspected {
            let child_end = run
                .events
                .iter()
                .position(|line| line == "frame_end CHILD InvalidFEOpcode")
                .expect("the child ends");
            let end = format!("call_end PARENT Return {}", Bytes::from_static(ON_RESUME));
            assert_eq!(
                &run.events[child_end + 1..],
                ["hook PARENT Return", &end, "frame_end PARENT Return"]
            );
        }
    }
}

/// A child the consumer starts from a frame's start runs before the frame does, and the frame
/// returns the action the consumer sets once the child's result is in it: the frame runs no
/// instruction of its own, on either path, and the inspector sees no step of it.
#[test]
fn test_a_child_started_at_a_frames_start_runs_before_the_frame() {
    let plan = Plan {
        at_start: Some(AtStart::Create),
        ..Plan::default()
    };
    for inspected in [false, true] {
        let run = run(plan, inspected);

        assert_eq!(
            run.output(),
            &status_and(true, AFTER_ITS_CHILD),
            "{inspected}"
        );
        assert_eq!(run.marked, (true, false), "the child ran no instruction");
        assert_eq!(
            run.hooks(),
            [
                "hook CREATION Return",
                "hook CHILD Return",
                "hook PARENT Return"
            ],
            "{inspected}"
        );
        if inspected {
            assert_eq!(run.steps("CHILD"), Vec::<&str>::new());
            let created = run
                .events
                .iter()
                .position(|line| line == "frame_end CREATION Return");
            let child_ends = run
                .events
                .iter()
                .position(|line| line == "frame_end CHILD Return");
            assert!(
                created < child_ends,
                "the creation ends first: {:?}",
                run.events
            );
        }
    }
}

/// A frame that resumes with no action set, from a child the consumer started at its start, runs
/// its code then, on either path; the inspector sees a step before each instruction, though the
/// frame was not handed to `initialize_interp`.
#[test]
fn test_a_frame_resumed_from_a_child_started_at_its_start_runs_its_code() {
    let plan = Plan {
        at_start: Some(AtStart::CreateThenRun),
        ..Plan::default()
    };
    for inspected in [false, true] {
        let run = run(plan, inspected);

        assert_eq!(run.output(), &status_and(false, &[]), "{inspected}");
        assert_eq!(
            run.hooks(),
            [
                "hook CREATION Return",
                "hook CHILD InvalidFEOpcode",
                "hook PARENT Return"
            ],
            "{inspected}"
        );
        if inspected {
            let steps = [
                "step CHILD 0",
                "step CHILD 2",
                "step CHILD 3",
                "step CHILD 4",
            ];
            assert_eq!(run.steps("CHILD"), steps, "PUSH1 PUSH0 SSTORE INVALID");
            assert_eq!(
                run.lines(|line| line.contains("CHILD") && !line.starts_with("step")),
                [
                    "hook CHILD InvalidFEOpcode",
                    "call_end CHILD InvalidFEOpcode 0x",
                    "frame_end CHILD InvalidFEOpcode"
                ]
            );
        }
    }
}

/// The result of each frame that ran reaches the consumer once, as it leaves the frame's run, on
/// both paths; on the inspected path, before the inspector's `call_end` and `frame_end`.
#[test]
fn test_a_frames_run_result_reaches_the_consumer_once_before_the_inspector() {
    for inspected in [false, true] {
        let run = run(Plan::default(), inspected);

        assert_eq!(
            run.hooks(),
            ["hook CHILD InvalidFEOpcode", "hook PARENT Return"],
            "{inspected}"
        );
        if inspected {
            let parent_end = format!("call_end PARENT Return {}", status_and(false, &[]));
            assert_eq!(
                run.ends(),
                [
                    "initialize_interp PARENT",
                    "initialize_interp CHILD",
                    "hook CHILD InvalidFEOpcode",
                    "call_end CHILD InvalidFEOpcode 0x",
                    "frame_end CHILD InvalidFEOpcode",
                    "hook PARENT Return",
                    &parent_end,
                    "frame_end PARENT Return",
                ]
            );
        }
    }
}

/// What the consumer leaves in a frame's result is what the inspector and the caller see: the
/// child's halt, made a revert with output, reaches the inspector's ends and the caller's status
/// and return data as that revert.
#[test]
fn test_the_consumers_rewrite_is_what_the_inspector_and_the_caller_see() {
    let plan = Plan {
        rewrite: true,
        ..Plan::default()
    };
    for inspected in [false, true] {
        let run = run(plan, inspected);

        assert_eq!(run.output(), &status_and(false, REWRITTEN), "{inspected}");
        assert_eq!(
            run.marked,
            (true, false),
            "the revert took the child's mark back"
        );
        if inspected {
            let child_end = format!("call_end CHILD Revert {}", Bytes::from_static(REWRITTEN));
            let seen = run.ends();
            let from = seen
                .iter()
                .position(|line| *line == "hook CHILD InvalidFEOpcode")
                .expect("the hook saw the halt");
            assert_eq!(
                seen[from + 1..from + 3],
                [&child_end, "frame_end CHILD Revert"]
            );
        }
    }
}

/// What the consumer leaves in the transaction's own frame's result is the transaction's result: a
/// transaction to [`CHILD`], whose halt the consumer makes a revert with output, ends as that
/// revert on both paths, and gets back the gas the frame left, as a revert does and a halt does
/// not.
#[test]
fn test_the_consumers_rewrite_of_the_transactions_frame_is_the_transactions_result() {
    let plan = Plan {
        rewrite: true,
        ..Plan::default()
    };
    for inspected in [false, true] {
        let halted = run_to(CHILD, Callee::Child, Plan::default(), inspected);
        let rewritten = run_to(CHILD, Callee::Child, plan, inspected);

        assert_eq!(
            halted.result.tx_gas_used(),
            GAS_LIMIT,
            "a halt spends it all"
        );
        let ExecutionResult::Revert { output, .. } = &rewritten.result else {
            panic!("the transaction reverts, got {:?}", rewritten.result);
        };
        assert_eq!(output, &Bytes::from_static(REWRITTEN), "{inspected}");
        // 21,000 for the transaction, then PUSH1 3, PUSH0 2 and SSTORE's cold zero-to-one 22,100.
        assert_eq!(rewritten.result.tx_gas_used(), 43_105, "{inspected}");
        assert_eq!(
            rewritten.marked,
            (false, false),
            "the halt took the mark back"
        );
        if inspected {
            let end = format!("call_end CHILD Revert {}", Bytes::from_static(REWRITTEN));
            assert_eq!(
                rewritten.ends(),
                [
                    "initialize_interp CHILD",
                    "hook CHILD InvalidFEOpcode",
                    &end,
                    "frame_end CHILD Revert"
                ]
            );
        }
    }
}

/// A result given without a run does not reach the consumer: neither a precompile's answer, which
/// revm gives at the frame's start, nor an inspector's.
#[test]
fn test_a_result_given_without_a_run_does_not_reach_the_consumer() {
    for inspected in [false, true] {
        let run = run_calling(Callee::Precompile, Plan::default(), inspected);

        assert_eq!(run.output(), &status_and(true, &[]), "{inspected}");
        assert_eq!(run.hooks(), ["hook PARENT Return"], "{inspected}");
    }

    let run = run_calling(Callee::Answered, Plan::default(), true);

    assert_eq!(run.output(), &status_and(false, ANSWERED));
    assert_eq!(run.hooks(), ["hook PARENT Return"]);
}
