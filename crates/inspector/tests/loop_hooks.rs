//! The two points the handler loops give a consumer's EVM, on the plain and the inspected path.
//!
//! A pending action: a consumer sets a frame's next action on its interpreter before the frame
//! runs, at its start or when a child returned into it, and the frame's run returns that action
//! before any instruction runs, without an inspector `step`.
//!
//! A frame's run result: the result of a frame that ran reaches the consumer's
//! [`EvmTr::frame_run_result`] once, as it leaves the frame's run, before the inspector's
//! `call_end` and `frame_end` and before its caller; what the consumer leaves in it is what they
//! see. A result given without a run (a precompile's, an inspector's answer) does not reach it.
//!
//! The consumer here wraps revm's EVM and changes nothing else: [`PARENT`] calls [`CHILD`], then
//! marks a slot of its own, and returns the call's status and return data; [`CHILD`] marks a slot
//! of its own and halts. A slot left unmarked is an instruction that did not run. The consumer's
//! hook writes its line into the inspector's, so one list orders both.

use context::{
    result::{EVMError, ExecutionResult, Output},
    Context, ContextSetters, ContextTr, Evm, FrameStack, JournalTr, TxEnv,
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
    CallInputs, CallOutcome, FrameInput, Gas, InputsImpl, InstructionResult, Interpreter,
    InterpreterAction, InterpreterResult, SharedMemory,
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

/// The output of the action the consumer sets on [`CHILD`] at its start.
const AT_START: &[u8] = b"set at its start";
/// The output of the action the consumer sets on [`PARENT`] when [`CHILD`] returns into it.
const ON_RESUME: &[u8] = b"set on its resume";
/// The output of the revert the consumer's hook makes of [`CHILD`]'s halt.
const REWRITTEN: &[u8] = b"rewritten as it leaves its run";
/// The output of the revert the inspector answers the call to [`CHILD`] with.
const ANSWERED: &[u8] = b"answered by the inspector";

/// What the consumer does at the loops' points.
#[derive(Clone, Copy, Debug, Default)]
struct Plan {
    /// A return carrying [`AT_START`] on [`CHILD`]'s frame, once revm built it.
    at_start: bool,
    /// A return carrying [`ON_RESUME`] on [`PARENT`]'s frame, once [`CHILD`]'s result is in it.
    on_resume: bool,
    /// [`CHILD`]'s halt made a revert carrying [`REWRITTEN`] as it leaves its run.
    rewrite: bool,
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
}

/// What the inspector saw, one line per callback, by the account the frame runs as.
#[derive(Debug, Default)]
struct Recorder {
    events: Vec<String>,
    /// Answers the call to [`CHILD`].
    answer: bool,
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
        _ => "other",
    }
}

impl Inspector<Ctx> for Recorder {
    fn initialize_interp(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        let account = name(interp.input.target_address);
        self.events.push(format!("initialize_interp {account}"));
    }

    fn step(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        let (account, pc) = (name(interp.input.target_address), interp.bytecode.pc());
        self.events.push(format!("step {account} {pc}"));
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
        let child = target(&frame_input.frame_input) == "CHILD";
        let mut started = self.evm.frame_init(frame_input)?;
        if let ItemOrResult::Item(frame) = &mut started {
            if child && self.plan.at_start {
                let gas = frame.interpreter.gas;
                frame
                    .interpreter
                    .bytecode
                    .set_action(InterpreterAction::new_return(
                        InstructionResult::Return,
                        Bytes::from_static(AT_START),
                        gas,
                    ));
            }
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
        if returned.is_none() && self.plan.on_resume {
            let caller = self.evm.frame_stack.get();
            let gas = caller.interpreter.gas;
            caller
                .interpreter
                .bytecode
                .set_action(InterpreterAction::new_return(
                    InstructionResult::Return,
                    Bytes::from_static(ON_RESUME),
                    gas,
                ));
        }
        Ok(returned)
    }

    /// Writes the result it got into the inspector's lines, and rewrites [`CHILD`]'s halt as
    /// [`Plan::rewrite`] says.
    fn frame_run_result(&mut self, result: &mut FrameResult) -> Result<(), ContextDbError<Ctx>> {
        let frame = self.evm.frame_stack.get();
        assert!(frame.is_finished(), "the frame on top is the one that ran");
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

/// What one transaction to [`PARENT`] did.
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
        self.events
            .iter()
            .map(String::as_str)
            .filter(|line| line.starts_with(&prefix))
            .collect()
    }

    /// The consumer's hook lines.
    fn hooks(&self) -> Vec<&str> {
        self.lines(|line| line.starts_with("hook "))
    }

    /// Every line but the steps: the frames' starts and ends, and the hook's.
    fn ends(&self) -> Vec<&str> {
        self.lines(|line| !line.starts_with("step"))
    }

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
    let (address, answer) = match callee {
        Callee::Child => (CHILD, false),
        Callee::Precompile => (IDENTITY, false),
        Callee::Answered => (CHILD, true),
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
            ..Recorder::default()
        }),
        plan,
    };
    consumer.evm.ctx.set_tx(
        TxEnv::builder_for_bench()
            .kind(TxKind::Call(PARENT))
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
/// run, on either path, and the inspector sees the frame start and end with no step between.
#[test]
fn test_an_action_set_at_a_frames_start_is_its_result() {
    let plan = Plan {
        at_start: true,
        ..Plan::default()
    };
    for inspected in [false, true] {
        let run = run(plan, inspected);

        assert_eq!(run.output(), &status_and(true, AT_START), "{inspected}");
        assert_eq!(run.marked, (true, false), "the child ran no instruction");
        assert_eq!(run.hooks(), ["hook CHILD Return", "hook PARENT Return"]);
        if inspected {
            let seen: Vec<&str> = run
                .events
                .iter()
                .map(String::as_str)
                .filter(|line| line.contains("CHILD"))
                .collect();
            let end = format!("call_end CHILD Return {}", Bytes::from_static(AT_START));
            assert_eq!(
                seen,
                [
                    "initialize_interp CHILD",
                    "hook CHILD Return",
                    &end,
                    "frame_end CHILD Return"
                ]
            );
        }
    }
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
