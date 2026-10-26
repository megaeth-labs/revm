//! The start lifecycle on revm's own runners, plain and inspected: every start revm opens settles
//! once, innermost first, and the transaction's boundary ends the lifecycle.
//!
//! The tests of the record and of the fatal errors run in every build. The tests that read the
//! events the hooks received need the lifecycle mirror the context keeps in debug builds only.
//!
//! [`PARENT`] calls [`CHILD`] (or the callee a test names) and then writes its slot 0; [`CHILD`]
//! writes its slot 0.

use context::{
    context_interface::context::ContextError,
    result::{EVMError, HaltReason},
    Context, ContextSetters, ContextTr, Evm, JournalTr, LocalContextTr, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    execution::settle_first_frame, instructions::EthInstructions, lifecycle, EthFrame,
    EthPrecompiles, EvmTr, ExecuteEvm, FrameResult, Handler, ItemOrResult, MainBuilder,
    MainContext, MainnetContext, MainnetHandler,
};
use interpreter::{
    interpreter::EthInterpreter, interpreter_types::LoopControl, CallInputs, CallOutcome,
    FrameInput, Gas, GasTracker, InstructionResult, Interpreter, InterpreterAction,
    InterpreterResult,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_inspector::{InspectEvm, Inspector, InspectorHandler};
use state::{
    bytecode::opcode::{CALL, CREATE, PUSH0, PUSH1, PUSH20, PUSH3, SSTORE, STOP},
    AccountInfo, Bytecode,
};
use std::convert::Infallible;

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;
type MainEvm<I> = Evm<Ctx, I, EthInstructions<EthInterpreter, Ctx>, EthPrecompiles, EthFrame>;
type Error = EVMError<Infallible>;

const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
const CHILD: Address = address!("0x00000000000000000000000000000000000c0de1");
const CREATOR: Address = address!("0x00000000000000000000000000000000000c0de2");
const ECRECOVER: Address = address!("0x0000000000000000000000000000000000000001");
const GAS_LIMIT: u64 = 1_000_000;

/// What an inspector does to the run. Most modes are read by the event tests, which need a debug
/// build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(debug_assertions), allow(dead_code))]
enum Mode {
    Observe,
    /// In `step_end` after the `CALL`: a fresh interpreter in place of the running one, which
    /// returns at once.
    ReplaceInterpreter,
    /// In `step_end` after the `CALL`: the running frame's input depth rewritten, and the
    /// `NewFrame` replaced with a return.
    RewriteInputDepth,
    /// In `step_end` after the `CALL`: the `NewFrame`'s input dropped and a copy put in its place.
    ReplaceInput,
    /// In `call(CHILD)`: a transaction run on a clone of the running context.
    CloneSimulation,
    /// In `call(CHILD)`: a transaction run on an EVM of its own.
    IndependentEvm,
    /// In `call(CHILD)`, once: a fatal error recorded on the context, as a failed lookup records
    /// one.
    FatalChildError,
}

struct Edit {
    mode: Mode,
    done: bool,
}

impl Edit {
    const fn new(mode: Mode) -> Self {
        Self { mode, done: false }
    }
}

impl Inspector<Ctx> for Edit {
    fn step_end(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
        if self.done {
            return;
        }
        let Some(InterpreterAction::NewFrame(FrameInput::Call(inputs))) = interp.bytecode.action()
        else {
            return;
        };
        let copy = (**inputs).clone();
        let ret = InterpreterAction::Return(InterpreterResult::new(
            InstructionResult::Return,
            Bytes::new(),
            Gas::new(0),
        ));
        match self.mode {
            Mode::ReplaceInterpreter => {
                *interp = Interpreter::default();
                interp.bytecode.set_action(ret);
            }
            Mode::RewriteInputDepth => {
                interp.input.depth += 1;
                *interp.bytecode.action() = Some(ret);
            }
            Mode::ReplaceInput => {
                drop(interp.bytecode.action().take());
                *interp.bytecode.action() = Some(InterpreterAction::NewFrame(FrameInput::Call(
                    Box::new(copy),
                )));
            }
            _ => return,
        }
        self.done = true;
    }

    fn call(&mut self, ctx: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        if inputs.target_address != CHILD || self.done {
            return None;
        }
        match self.mode {
            Mode::CloneSimulation => {
                let mut nested = ctx.clone().build_mainnet();
                let tx = precompile_tx().nonce(1).build_fill();
                assert!(nested
                    .transact(tx)
                    .expect("the simulation runs")
                    .result
                    .is_success());
            }
            Mode::IndependentEvm => {
                let mut nested = Context::mainnet().with_db(Db::default()).build_mainnet();
                let tx = precompile_tx().build_fill();
                assert!(nested
                    .transact(tx)
                    .expect("the nested EVM runs")
                    .result
                    .is_success());
            }
            Mode::FatalChildError => {
                *ctx.error() = Err(ContextError::Custom("injected".into()));
            }
            _ => return None,
        }
        self.done = true;
        None
    }
}

fn precompile_tx() -> context::tx::TxEnvBuilder {
    TxEnv::builder_for_bench()
        .kind(TxKind::Call(ECRECOVER))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
}

fn account(code: Vec<u8>) -> AccountInfo {
    let code = Bytecode::new_raw(code.into());
    AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code)
}

/// [`PARENT`] calling `callee`, [`CHILD`], and [`CREATOR`], which creates an empty contract.
fn db(callee: Address) -> Db {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    let mut parent = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20];
    parent.extend_from_slice(callee.as_slice());
    parent.extend_from_slice(&[PUSH3, 0x01, 0x00, 0x00, CALL, PUSH1, 1, PUSH0, SSTORE, STOP]);
    db.insert_account_info(PARENT, account(parent));
    db.insert_account_info(CHILD, account(vec![PUSH1, 2, PUSH0, SSTORE, STOP]));
    db.insert_account_info(CREATOR, account(vec![PUSH0, PUSH0, PUSH0, CREATE, STOP]));
    db
}

fn context(callee: Address, spec: SpecId) -> Ctx {
    Context::mainnet()
        .with_db(db(callee))
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(spec))
}

fn evm() -> MainEvm<()> {
    context(CHILD, SpecId::OSAKA).build_mainnet()
}

fn inspected(mode: Mode) -> MainEvm<Edit> {
    context(CHILD, SpecId::OSAKA).build_mainnet_with_inspector(Edit::new(mode))
}

fn tx(to: Address, nonce: u64) -> TxEnv {
    TxEnv::builder_for_bench()
        .kind(TxKind::Call(to))
        .nonce(nonce)
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill()
}

/// The record is as a transaction's boundary leaves it: no lifecycle under way, nothing open.
fn assert_clear<I>(evm: &mut MainEvm<I>) {
    assert!(
        !evm.frame_stack.lifecycle_active(),
        "a lifecycle is still under way"
    );
    assert!(evm.frame_stack.lifecycle_settled(), "a start is still open");
}

fn assert_fatal<T: core::fmt::Debug>(result: Result<T, Error>, message: &str) {
    match result {
        Err(EVMError::Custom(error)) => assert!(error.contains(message), "{error}"),
        other => panic!("expected the fatal error {message:?}, got {other:?}"),
    }
}

/// Every transaction a reused EVM runs, plain and inspected, starts from a clear record and
/// leaves one. (`transact` commits nothing to the database, so each runs at nonce 0.)
#[test]
fn test_a_reused_evm_runs_each_transaction_on_a_clear_record() {
    let mut evm = inspected(Mode::Observe);
    for _ in 0..3 {
        assert!(evm
            .transact(tx(PARENT, 0))
            .expect("runs")
            .result
            .is_success());
        assert_clear(&mut evm);
        assert!(evm
            .inspect_tx(tx(PARENT, 0))
            .expect("runs")
            .result
            .is_success());
        assert_clear(&mut evm);
    }
}

/// A fatal error in a child discards the lifecycle at the handler's fatal boundary, and the EVM
/// runs the next transaction.
#[test]
fn test_a_fatal_child_error_discards_the_lifecycle_and_the_evm_is_reused() {
    let mut evm = inspected(Mode::FatalChildError);
    assert_fatal(evm.inspect_tx(tx(PARENT, 0)), "injected");
    assert_clear(&mut evm);
    assert!(evm
        .inspect_tx(tx(PARENT, 0))
        .expect("reused")
        .result
        .is_success());
    assert_clear(&mut evm);
}

/// A caller of `run_without_catch_error` that drops an error makes the cleanup `catch_error`
/// makes before it reuses the EVM: discarding the lifecycle alone leaves the frame stack's index,
/// so the next transaction's start would open under a frame with no start.
#[test]
fn test_reuse_after_an_uncaught_error_needs_the_frame_stack_cleared() {
    let fail = |evm: &mut MainEvm<Edit>| {
        evm.inspector.done = false;
        evm.ctx.set_tx(tx(PARENT, 0));
        let mut handler = MainnetHandler::<_, Error, EthFrame>::default();
        assert!(handler.inspect_run_without_catch_error(evm).is_err());
        assert!(
            evm.frame_stack.lifecycle_active(),
            "nothing discarded the lifecycle"
        );
    };

    // With nothing cleaned up, the journal still holds the dropped transaction's nonce.
    let mut evm = inspected(Mode::FatalChildError);
    fail(&mut evm);
    assert_fatal(
        evm.inspect_tx(tx(PARENT, 1)),
        "a start opened where one is still open",
    );

    let mut evm = inspected(Mode::FatalChildError);
    fail(&mut evm);
    lifecycle::discard_transaction(&mut evm);
    evm.ctx.journal_mut().discard_tx();
    assert_fatal(evm.inspect_tx(tx(PARENT, 0)), "under a frame with no start");

    let mut evm = inspected(Mode::FatalChildError);
    fail(&mut evm);
    lifecycle::discard_transaction(&mut evm);
    evm.ctx.journal_mut().discard_tx();
    evm.ctx.local_mut().clear();
    evm.frame_stack.clear();
    assert!(evm
        .inspect_tx(tx(PARENT, 0))
        .expect("reused")
        .result
        .is_success());
    assert_clear(&mut evm);
}

/// The mainnet handler with a `last_frame_result` that settles the transaction's frame but does
/// not close the transaction's start, as an override that misses the call would.
#[derive(Default)]
struct MissesTheClose;

impl Handler for MissesTheClose {
    type Evm = MainEvm<()>;
    type Error = Error;
    type HaltReason = HaltReason;

    fn last_frame_result(
        &mut self,
        evm: &mut Self::Evm,
        frame_result: &mut FrameResult,
        parent_gas: &mut GasTracker,
    ) -> Result<(), Self::Error> {
        settle_first_frame::<_, Self::Error>(evm.ctx(), frame_result, parent_gas)?;
        *frame_result.gas_mut().tracker_mut() = *parent_gas;
        Ok(())
    }
}

/// A missed settlement is a fatal error at the successful boundary, in every build, and the
/// boundary discards the lifecycle, so the EVM runs the next transaction.
#[test]
fn test_a_missed_settlement_fails_the_transaction_at_the_successful_boundary() {
    let mut evm = evm();
    evm.ctx.set_tx(tx(PARENT, 0));
    assert_fatal(
        MissesTheClose.run(&mut evm),
        "the transaction ended with a start open",
    );
    assert_clear(&mut evm);
    assert!(evm
        .transact(tx(PARENT, 0))
        .expect("reused")
        .result
        .is_success());
}

/// A runner that drives the transaction's frame itself without opening the transaction's start:
/// the frame's child has no parent, and the transaction's settlement finds no open start.
#[test]
fn test_a_runner_that_skips_the_transactions_start_is_refused() {
    let mut runner = evm();
    runner.ctx.set_tx(tx(PARENT, 0));
    let mut handler = MainnetHandler::<_, Error, EthFrame>::default();
    let mut gas = handler.tx_gas(&mut runner, &Default::default());
    let init = handler
        .first_frame_input(&mut runner, &mut gas)
        .expect("the frame's input")
        .expect("the runtime gas phase passes");
    let ItemOrResult::Item(_) = runner.frame_init(init).expect("the frame starts") else {
        panic!("PARENT runs");
    };
    let error = runner.frame_run().expect_err("the call opens no start");
    assert!(
        matches!(&error, ContextError::Custom(e) if e.contains("under a frame with no start")),
        "{error:?}"
    );

    let mut evm = evm();
    let mut result = FrameResult::new_call_oog(GAS_LIMIT, 0..0, 0);
    let mut gas = GasTracker::new(GAS_LIMIT, 0, 0);
    let error = handler
        .last_frame_result(&mut evm, &mut result, &mut gas)
        .expect_err("the transaction's start is not open");
    assert!(
        matches!(&error, EVMError::Custom(e) if e.contains("found no open start")),
        "{error:?}"
    );
}

/// A marked tracker and outcome compare, hash and print as the same values unmarked.
#[test]
fn test_the_marks_take_no_part_in_the_values() {
    use interpreter::{CreateOutcome, OutcomeMark, StartMark};
    use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
    let hash =
        |tracker: &GasTracker| BuildHasherDefault::<DefaultHasher>::default().hash_one(tracker);

    let plain = GasTracker::new(100, 90, 5);
    let mut marked = plain;
    marked.set_start_mark(StartMark::for_start(7));
    assert!(marked.start_mark().is_for(7));
    assert_eq!(marked, plain);
    assert_eq!(hash(&marked), hash(&plain));
    assert_eq!(format!("{marked:?}"), format!("{plain:?}"));

    let result = InterpreterResult::new(InstructionResult::Stop, Bytes::new(), Gas::new(10));
    let call = CallOutcome::new(result.clone(), 0..0);
    let marked_call = CallOutcome {
        start_mark: OutcomeMark::for_start(7),
        ..call.clone()
    };
    assert_eq!(marked_call, call);
    assert_eq!(format!("{marked_call:?}"), format!("{call:?}"));
    let create = CreateOutcome::new(result, None);
    let marked_create = CreateOutcome {
        start_mark: OutcomeMark::for_start(7),
        ..create.clone()
    };
    assert_eq!(marked_create, create);
    assert_eq!(format!("{marked_create:?}"), format!("{create:?}"));
    for printed in [
        format!("{marked:?}"),
        format!("{marked_call:?}"),
        format!("{marked_create:?}"),
    ] {
        assert!(!printed.contains("Mark"), "{printed}");
    }
}

/// An EVM on Amsterdam, where a value transfer to an empty account pays the account's state in the
/// transaction's runtime gas phase.
fn runtime_out_of_gas_evm() -> MainEvm<()> {
    Context::mainnet()
        .with_db(db(CHILD))
        .modify_cfg_chained(|cfg| {
            cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
            cfg.tx_gas_limit_cap = Some(u64::MAX);
        })
        .build_mainnet()
}

/// A value transfer to an empty account funded for its intrinsic gas only: valid, and out of gas
/// in the runtime gas phase.
fn runtime_out_of_gas_tx() -> TxEnv {
    use primitives::{eip2780, eip8038};
    TxEnv::builder_for_bench()
        .kind(TxKind::Call(address!(
            "0x00000000000000000000000000000000000000ab"
        )))
        .value(U256::from(1))
        .gas_price(0)
        .gas_limit(eip2780::TX_BASE_COST + eip8038::COLD_ACCOUNT_ACCESS + eip2780::TX_VALUE_COST)
        .build_fill()
}

/// The stand-in for the frame of a transaction whose runtime gas phase ran out of gas settles on
/// the transaction's start, so the transaction is included as a halt and leaves the record clear.
#[test]
fn test_the_runtime_out_of_gas_stand_in_leaves_the_record_clear() {
    let mut evm = runtime_out_of_gas_evm();
    for _ in 0..2 {
        assert!(evm
            .transact(runtime_out_of_gas_tx())
            .expect("included")
            .result
            .is_halt());
        assert_clear(&mut evm);
    }
}

/// The events the hooks received, which the mirror the context keeps in debug builds records.
#[cfg(debug_assertions)]
mod events {
    use super::*;
    use context::context_interface::{
        host::{FrameSplit, LifecycleEnd},
        FrameStack, Host, LifecycleEvent,
    };

    fn describe(event: &LifecycleEvent) -> String {
        match event {
            LifecycleEvent::Split(split) => format!(
                "split {} parent {:?} depth {}",
                split.id, split.parent, split.depth
            ),
            LifecycleEvent::Settle(settle) => {
                let mut line = format!(
                    "settle {} depth {} {:?}",
                    settle.id, settle.depth, settle.status
                );
                if let Some(created) = settle.created {
                    line.push_str(&format!(" created {created}"));
                }
                for (flag, name) in [
                    (settle.inspector_answer, "inspector-answer"),
                    (settle.gas_replaced, "gas-replaced"),
                    (settle.outcome_rebuilt, "outcome-rebuilt"),
                ] {
                    if flag {
                        line.push(' ');
                        line.push_str(name);
                    }
                }
                line
            }
            LifecycleEvent::End(end) => format!("end {end:?}"),
        }
    }

    pub(super) fn events<I>(evm: &mut MainEvm<I>) -> Vec<String> {
        let mirror = evm
            .ctx
            .local
            .lifecycle_mirror()
            .expect("a debug build keeps one");
        mirror.events().iter().map(describe).collect()
    }

    pub(super) const PAIRED: [&str; 5] = [
        "split 1 parent None depth 0",
        "split 2 parent Some(1) depth 1",
        "settle 2 depth 1 Ok",
        "settle 1 depth 0 Ok",
        "end Settled",
    ];

    const ROOT_ONLY: [&str; 3] = [
        "split 1 parent None depth 0",
        "settle 1 depth 0 Ok",
        "end Settled",
    ];

    fn inspect(mode: Mode, callee: Address) -> Vec<String> {
        let mut evm = context(callee, SpecId::OSAKA).build_mainnet_with_inspector(Edit::new(mode));
        let result = evm.inspect_tx(tx(PARENT, 0)).expect("runs");
        assert!(result.result.is_success(), "{mode:?}: {:?}", result.result);
        events(&mut evm)
    }

    #[test]
    fn test_a_plain_run_settles_one_start_per_frame() {
        let mut evm = evm();
        assert!(evm
            .transact(tx(PARENT, 0))
            .expect("runs")
            .result
            .is_success());
        assert_eq!(events(&mut evm), PAIRED);
    }

    #[test]
    fn test_an_observed_run_settles_one_start_per_frame() {
        assert_eq!(inspect(Mode::Observe, CHILD), PAIRED);
    }

    #[test]
    fn test_a_precompiles_answer_settles_the_start_it_answers() {
        assert_eq!(inspect(Mode::Observe, ECRECOVER), PAIRED);
    }

    #[test]
    fn test_a_replaced_interpreter_leaves_no_start_open() {
        assert_eq!(inspect(Mode::ReplaceInterpreter, CHILD), ROOT_ONLY);
    }

    #[test]
    fn test_a_rewritten_input_depth_changes_no_start() {
        assert_eq!(inspect(Mode::RewriteInputDepth, CHILD), ROOT_ONLY);
    }

    #[test]
    fn test_a_replaced_input_opens_its_own_start() {
        assert_eq!(inspect(Mode::ReplaceInput, CHILD), PAIRED);
    }

    #[test]
    fn test_a_simulation_on_a_cloned_context_leaves_the_outer_lifecycle_paired() {
        assert_eq!(inspect(Mode::CloneSimulation, CHILD), PAIRED);
    }

    #[test]
    fn test_an_independent_nested_evm_leaves_the_outer_lifecycle_paired() {
        assert_eq!(inspect(Mode::IndependentEvm, CHILD), PAIRED);
    }

    #[test]
    fn test_a_fatal_child_error_ends_the_lifecycle_discarded() {
        let mut evm = inspected(Mode::FatalChildError);
        assert!(evm.inspect_tx(tx(PARENT, 0)).is_err());
        assert_eq!(
            events(&mut evm),
            [
                "split 1 parent None depth 0",
                "split 2 parent Some(1) depth 1",
                "end Discarded",
            ]
        );
        assert!(evm
            .inspect_tx(tx(PARENT, 0))
            .expect("reused")
            .result
            .is_success());
        assert_eq!(events(&mut evm), PAIRED);
    }

    #[test]
    fn test_a_creation_reports_the_address_it_took() {
        let mut evm = evm();
        assert!(evm
            .transact(tx(CREATOR, 0))
            .expect("runs")
            .result
            .is_success());
        assert_eq!(
            events(&mut evm),
            [
                "split 1 parent None depth 0",
                "split 2 parent Some(1) depth 1",
                "settle 2 depth 1 Ok created true",
                "settle 1 depth 0 Ok",
                "end Settled",
            ]
        );
        let creation = TxEnv::builder_for_bench()
            .kind(TxKind::Create)
            .data(Bytes::from_static(&[STOP]))
            .gas_price(0)
            .gas_limit(GAS_LIMIT)
            .build_fill();
        assert!(evm.transact(creation).expect("runs").result.is_success());
        assert_eq!(
            events(&mut evm),
            [
                "split 1 parent None depth 0",
                "settle 1 depth 0 Ok created true",
                "end Settled",
            ]
        );
    }

    /// A transaction whose runtime gas phase runs out of gas settles the stand-in for its frame
    /// on the transaction's start.
    #[test]
    fn test_the_runtime_out_of_gas_stand_in_settles_the_transactions_start() {
        let mut evm = runtime_out_of_gas_evm();
        assert!(evm
            .transact(runtime_out_of_gas_tx())
            .expect("included")
            .result
            .is_halt());
        assert_eq!(
            events(&mut evm),
            [
                "split 1 parent None depth 0",
                "settle 1 depth 0 Halt",
                "end Settled",
            ]
        );
    }

    #[test]
    fn test_a_system_call_settles_every_start() {
        use handler::SystemCallEvm;
        let mut evm = evm();
        let result = evm
            .system_call_one(PARENT, Bytes::new())
            .expect("the system call runs");
        assert!(result.is_success());
        assert_eq!(events(&mut evm), PAIRED);
    }

    /// What an inspector does to a start's result, which revm reports on the settlement.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Flag {
        /// `call` answers with an outcome a constructor built.
        Answer,
        /// `call_end` puts a constructor-built tracker in the result's gas.
        ReplaceGas,
        /// `call_end` restores the tracker it saved when the frame's interpreter started.
        RestoreSnapshot,
        /// `call_end` rewrites the status only.
        RewriteStatus,
        /// `call_end` builds a new outcome around the same result.
        RebuildOutcome,
        /// `call_end` rebuilds the outcome and its result, keeping the outcome's other fields and
        /// the result's gas.
        KeepOutcome,
        /// `create_end` builds a new outcome around the same result and address.
        RebuildCreation,
    }

    struct Rewrite {
        flag: Flag,
        target: Address,
        snapshot: Option<Gas>,
    }

    impl Inspector<Ctx> for Rewrite {
        fn initialize_interp(&mut self, interp: &mut Interpreter, _: &mut Ctx) {
            if interp.input.target_address == self.target {
                self.snapshot = Some(interp.gas);
            }
        }

        fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
            (self.flag == Flag::Answer && inputs.target_address == self.target).then(|| {
                CallOutcome::new(
                    InterpreterResult::new(
                        InstructionResult::Return,
                        Bytes::new(),
                        Gas::new(inputs.gas_limit),
                    ),
                    inputs.return_memory_offset.clone(),
                )
            })
        }

        fn call_end(&mut self, _: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
            if inputs.target_address != self.target {
                return;
            }
            match self.flag {
                Flag::ReplaceGas => outcome.result.gas = Gas::new(inputs.gas_limit),
                Flag::RestoreSnapshot => {
                    outcome.result.gas = self.snapshot.expect("the frame's interpreter started");
                }
                Flag::RewriteStatus => outcome.result.result = InstructionResult::Revert,
                Flag::RebuildOutcome => {
                    *outcome =
                        CallOutcome::new(outcome.result.clone(), outcome.memory_offset.clone());
                }
                Flag::KeepOutcome => {
                    let result = InterpreterResult::new(
                        outcome.result.result,
                        outcome.result.output.clone(),
                        outcome.result.gas,
                    );
                    *outcome = CallOutcome {
                        result,
                        ..outcome.clone()
                    };
                }
                Flag::Answer | Flag::RebuildCreation => {}
            }
        }

        fn create_end(
            &mut self,
            _: &mut Ctx,
            _: &interpreter::CreateInputs,
            outcome: &mut interpreter::CreateOutcome,
        ) {
            if self.flag == Flag::RebuildCreation {
                *outcome = interpreter::CreateOutcome::new(outcome.result.clone(), outcome.address);
            }
        }
    }

    /// The events of [`PARENT`]'s transaction, calling `callee`, under `flag`, or of
    /// [`CREATOR`]'s for a creation's flag.
    fn flagged(flag: Flag, target: Address, callee: Address) -> Vec<String> {
        let mut evm = context(callee, SpecId::OSAKA).build_mainnet_with_inspector(Rewrite {
            flag,
            target,
            snapshot: None,
        });
        let to = if flag == Flag::RebuildCreation {
            CREATOR
        } else {
            PARENT
        };
        let result = evm.inspect_tx(tx(to, 0)).expect("runs");
        assert!(result.result.is_success(), "{flag:?}: {:?}", result.result);
        events(&mut evm)
    }

    /// [`PAIRED`] with the child's settlement as `child`.
    fn paired_with(child: &str) -> Vec<String> {
        let mut events = PAIRED.map(String::from).to_vec();
        events[2] = child.to_string();
        events
    }

    #[test]
    fn test_an_inspectors_answer_is_flagged() {
        assert_eq!(
            flagged(Flag::Answer, CHILD, CHILD),
            paired_with("settle 2 depth 1 Ok inspector-answer")
        );
    }

    #[test]
    fn test_an_inspectors_answer_to_the_transactions_frame_is_flagged() {
        assert_eq!(
            flagged(Flag::Answer, PARENT, CHILD),
            [
                "split 1 parent None depth 0",
                "settle 1 depth 0 Ok inspector-answer",
                "end Settled",
            ]
        );
    }

    #[test]
    fn test_a_replaced_tracker_is_flagged() {
        assert_eq!(
            flagged(Flag::ReplaceGas, CHILD, CHILD),
            paired_with("settle 2 depth 1 Ok gas-replaced")
        );
        assert_eq!(
            flagged(Flag::ReplaceGas, ECRECOVER, ECRECOVER),
            paired_with("settle 2 depth 1 Ok gas-replaced"),
            "a precompile's answer"
        );
    }

    /// A snapshot taken before revm marked the result is a replacement, even of the same
    /// start's tracker.
    #[test]
    fn test_a_restored_snapshot_is_flagged() {
        assert_eq!(
            flagged(Flag::RestoreSnapshot, CHILD, CHILD),
            paired_with("settle 2 depth 1 Ok gas-replaced")
        );
    }

    #[test]
    fn test_a_status_only_rewrite_is_not_flagged() {
        assert_eq!(
            flagged(Flag::RewriteStatus, CHILD, CHILD),
            paired_with("settle 2 depth 1 Revert")
        );
    }

    #[test]
    fn test_an_outcome_rebuilt_around_the_same_result_is_flagged() {
        assert_eq!(
            flagged(Flag::RebuildOutcome, CHILD, CHILD),
            paired_with("settle 2 depth 1 Ok outcome-rebuilt")
        );
        assert_eq!(
            flagged(Flag::RebuildCreation, CHILD, CHILD),
            [
                "split 1 parent None depth 0",
                "split 2 parent Some(1) depth 1",
                "settle 2 depth 1 Ok created true outcome-rebuilt",
                "settle 1 depth 0 Ok",
                "end Settled",
            ]
        );
    }

    #[test]
    fn test_a_rebuild_that_keeps_the_outcome_and_the_gas_is_not_flagged() {
        assert_eq!(flagged(Flag::KeepOutcome, CHILD, CHILD), PAIRED);
    }

    /// The context hands every event to its mirror: one that does not fit fails the check.
    #[test]
    #[should_panic(expected = "the transaction's start 1 at depth 5")]
    fn test_the_contexts_hooks_feed_the_mirror() {
        let mut ctx = context(CHILD, SpecId::OSAKA);
        let mut stack = FrameStack::<EthFrame>::new();
        let split: FrameSplit = stack.open_start(5).expect("the record opens it");
        ctx.frame_split(&split);
        ctx.frame_lifecycle_end(LifecycleEnd::Settled);
    }
}
