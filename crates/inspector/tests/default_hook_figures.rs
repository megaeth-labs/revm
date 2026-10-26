//! With the default hooks, the start lifecycle changes no figure revm's settlement computes.
//!
//! Each case answers or rewrites a start's result the way an inspector or a consumer's
//! `frame_init` can, on the transaction's frame or on a child, and the transaction's figures must
//! be the ones revm computed before the lifecycle existed. The expected figures were recorded on
//! that revision (`b370b538`), so a lifecycle helper that moves a figure fails here.
//!
//! The transaction runs on Amsterdam with every gas regular (the cap is lifted), so the
//! transaction's frame starts with no reservoir and every state charge spills into regular gas:
//! a result that hands a reservoir back then unwinds the spill into regular gas, and a halt keeps
//! that gas. [`EMPTY`] is an account with no code; a value transfer to it pays for its leaf, as
//! a creation pays for the account it creates.

use context::{
    result::{EVMError, ExecutionResult},
    Context, ContextSetters, Evm, FrameStack, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    evm::{ContextDbError, FrameInitResult},
    instructions::EthInstructions,
    EthFrame, EthPrecompiles, EvmTr, ExecuteEvm, FrameInitOrResult, FrameResult, Handler,
    ItemOrResult, MainBuilder, MainContext, MainnetContext, MainnetHandler,
};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, CallInputs, CallOutcome,
    CreateInputs, CreateOutcome, FrameInput, Gas, InstructionResult, InterpreterResult,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_inspector::{InspectEvm, Inspector};
use state::{
    bytecode::opcode::{CALL, CREATE, PUSH0, PUSH1, PUSH20, PUSH3, STOP},
    AccountInfo, Bytecode,
};
use std::convert::Infallible;

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;
type MainEvm<I> = Evm<Ctx, I, EthInstructions<EthInterpreter, Ctx>, EthPrecompiles, EthFrame>;
type Error = EVMError<Infallible>;

/// Calls [`EMPTY`] with a value of 1 and stops.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Creates an empty contract and stops.
const CREATOR: Address = address!("0x00000000000000000000000000000000000c0de2");
/// No code, no balance, no nonce.
const EMPTY: Address = address!("0x00000000000000000000000000000000000000ab");
const GAS_LIMIT: u64 = 1_000_000;

/// The frame a case acts on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Target {
    /// The transaction's frame: a value transfer to [`EMPTY`].
    Root,
    /// [`PARENT`]'s call to [`EMPTY`], at depth 1.
    Child,
    /// The transaction's frame: a creation with no init code.
    RootCreation,
    /// [`CREATOR`]'s creation, at depth 1.
    ChildCreation,
}

impl Target {
    /// The caller of the target's start.
    const fn caller(self) -> Address {
        match self {
            Self::Root | Self::RootCreation => BENCH_CALLER,
            Self::Child => PARENT,
            Self::ChildCreation => CREATOR,
        }
    }

    const fn is_creation(self) -> bool {
        matches!(self, Self::RootCreation | Self::ChildCreation)
    }
}

/// How much of the start's gas limit an answer's tracker has left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Left {
    All,
    Half,
}

/// What is done to the start's result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// The start is answered with a constructor-built outcome: `status`, a tracker with `left` of
    /// the gas limit and a reservoir of the given amount.
    Answer(InstructionResult, Left, u64),
    /// The end callback puts a constructor-built tracker in the result, with `left` of the gas
    /// limit and a reservoir of the given amount.
    ReplaceGas(Left, u64),
    /// The end callback builds a new outcome around the same result.
    RebuildOutcome,
    /// The end callback rewrites the status only.
    RewriteStatus(InstructionResult),
    /// The end callback rewrites the status and puts a constructor-built tracker with the whole
    /// gas limit and a reservoir of the given amount in the result.
    RewriteAndReplace(InstructionResult, u64),
}

/// A tracker built as an answer builds one: `left` of `gas_limit` and `reservoir`.
fn built_gas(gas_limit: u64, left: Left, reservoir: u64) -> Gas {
    let mut gas = Gas::new(gas_limit);
    if left == Left::Half {
        gas.tracker_mut().set_remaining(gas_limit / 2);
    }
    gas.tracker_mut().set_reservoir(reservoir);
    gas
}

/// Whether the start of `caller` is the target's.
fn targeted(target: Target, caller: Address, creation: bool) -> bool {
    caller == target.caller() && creation == target.is_creation()
}

/// `shape`'s end-callback edit of a result whose start had `gas_limit`.
fn edit(shape: Shape, gas_limit: u64, result: &mut InterpreterResult) {
    match shape {
        Shape::Answer(..) | Shape::RebuildOutcome => {}
        Shape::ReplaceGas(left, reservoir) => result.gas = built_gas(gas_limit, left, reservoir),
        Shape::RewriteStatus(status) => result.result = status,
        Shape::RewriteAndReplace(status, reservoir) => {
            result.result = status;
            result.gas = built_gas(gas_limit, Left::All, reservoir);
        }
    }
}

/// Acts on the target's start as an inspector.
struct Rewrites(Target, Shape);

/// The answer `shape` makes to a start with `gas_limit`, if it answers.
fn answer(shape: Shape, gas_limit: u64) -> Option<InterpreterResult> {
    let Shape::Answer(status, left, reservoir) = shape else {
        return None;
    };
    let gas = built_gas(gas_limit, left, reservoir);
    Some(InterpreterResult::new(status, Bytes::new(), gas))
}

impl Inspector<Ctx> for Rewrites {
    fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        if !targeted(self.0, inputs.caller, false) {
            return None;
        }
        let result = answer(self.1, inputs.gas_limit)?;
        Some(CallOutcome::new(
            result,
            inputs.return_memory_offset.clone(),
        ))
    }

    fn call_end(&mut self, _: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
        if !targeted(self.0, inputs.caller, false) {
            return;
        }
        if self.1 == Shape::RebuildOutcome {
            *outcome = CallOutcome::new(outcome.result.clone(), outcome.memory_offset.clone());
        }
        edit(self.1, inputs.gas_limit, &mut outcome.result);
    }

    fn create(&mut self, _: &mut Ctx, inputs: &mut CreateInputs) -> Option<CreateOutcome> {
        if !targeted(self.0, inputs.caller(), true) {
            return None;
        }
        Some(CreateOutcome::new(
            answer(self.1, inputs.gas_limit())?,
            None,
        ))
    }

    fn create_end(&mut self, _: &mut Ctx, inputs: &CreateInputs, outcome: &mut CreateOutcome) {
        if !targeted(self.0, inputs.caller(), true) {
            return;
        }
        if self.1 == Shape::RebuildOutcome {
            *outcome = CreateOutcome::new(outcome.result.clone(), outcome.address);
        }
        edit(self.1, inputs.gas_limit(), &mut outcome.result);
    }
}

/// revm's mainnet EVM whose `frame_init` answers the target's start, as a consumer's
/// `frame_init` answers a start it intercepts.
struct Answers {
    evm: MainEvm<()>,
    target: Target,
    answer: (InstructionResult, Left, u64),
}

impl EvmTr for Answers {
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
        frame_init: FrameInit,
    ) -> Result<FrameInitResult<'_, EthFrame>, ContextDbError<Ctx>> {
        let (status, left, reservoir) = self.answer;
        let shape = Shape::Answer(status, left, reservoir);
        match &frame_init.frame_input {
            FrameInput::Call(inputs) if targeted(self.target, inputs.caller, false) => {
                let result = answer(shape, inputs.gas_limit).expect("an answer");
                let offset = inputs.return_memory_offset.clone();
                Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome::new(
                    result, offset,
                ))))
            }
            FrameInput::Create(inputs) if targeted(self.target, inputs.caller(), true) => {
                let result = answer(shape, inputs.gas_limit()).expect("an answer");
                Ok(ItemOrResult::Result(FrameResult::Create(
                    CreateOutcome::new(result, None),
                )))
            }
            _ => self.evm.frame_init(frame_init),
        }
    }

    fn frame_run(&mut self) -> Result<FrameInitOrResult<EthFrame>, ContextDbError<Ctx>> {
        self.evm.frame_run()
    }

    fn frame_return_result(
        &mut self,
        result: FrameResult,
    ) -> Result<Option<FrameResult>, ContextDbError<Ctx>> {
        self.evm.frame_return_result(result)
    }
}

fn account(code: Vec<u8>, balance: u64) -> AccountInfo {
    let code = Bytecode::new_raw(code.into());
    AccountInfo::new(U256::from(balance), 1, code.hash_slow(), code)
}

fn context() -> Ctx {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    // PUSH0 PUSH0 PUSH0 PUSH0 (no data in or out), PUSH1 1 (the value), PUSH20 EMPTY,
    // PUSH3 0x010000 (the gas), CALL, STOP.
    let mut parent = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH1, 1, PUSH20];
    parent.extend_from_slice(EMPTY.as_slice());
    parent.extend_from_slice(&[PUSH3, 0x01, 0x00, 0x00, CALL, STOP]);
    db.insert_account_info(PARENT, account(parent, 1));
    db.insert_account_info(CREATOR, account(vec![PUSH0, PUSH0, PUSH0, CREATE, STOP], 0));
    Context::mainnet().with_db(db).modify_cfg_chained(|cfg| {
        cfg.set_spec_and_mainnet_gas_params(SpecId::AMSTERDAM);
        cfg.tx_gas_limit_cap = Some(u64::MAX);
    })
}

fn tx(target: Target) -> TxEnv {
    let (kind, value) = match target {
        Target::Root => (TxKind::Call(EMPTY), 1),
        Target::Child => (TxKind::Call(PARENT), 0),
        Target::RootCreation => (TxKind::Create, 0),
        Target::ChildCreation => (TxKind::Call(CREATOR), 0),
    };
    TxEnv::builder_for_bench()
        .kind(kind)
        .value(U256::from(value))
        .gas_price(0)
        .gas_limit(GAS_LIMIT)
        .build_fill()
}

/// The figures a transaction's result carries: its kind, its gas and its output.
fn figures(result: &ExecutionResult) -> String {
    let kind = match result {
        ExecutionResult::Success { reason, .. } => format!("Success({reason:?})"),
        ExecutionResult::Revert { .. } => "Revert".to_string(),
        ExecutionResult::Halt { reason, .. } => format!("Halt({reason:?})"),
    };
    let gas = result.gas();
    format!(
        "{kind} spent {} state {} refunded {} floor {} output {}",
        gas.total_gas_spent(),
        gas.state_gas_spent_final(),
        gas.inner_refunded(),
        gas.floor_gas(),
        result.output().cloned().unwrap_or_default(),
    )
}

/// The transaction of `target`, run plain.
fn plain(target: Target) -> String {
    let result = context()
        .build_mainnet()
        .transact(tx(target))
        .expect("runs");
    figures(&result.result)
}

/// The transaction of `target`, inspected under `shape`.
fn inspected(target: Target, shape: Shape) -> String {
    let mut evm = context().build_mainnet_with_inspector(Rewrites(target, shape));
    let result = evm.inspect_tx(tx(target)).expect("runs");
    figures(&result.result)
}

/// The transaction of `target`, its start answered by the consumer's `frame_init`.
fn answered(target: Target, answer: (InstructionResult, Left, u64)) -> String {
    let mut evm = Answers {
        evm: context().build_mainnet(),
        target,
        answer,
    };
    evm.evm.ctx.set_tx(tx(target));
    let result = MainnetHandler::<_, Error, EthFrame>::default()
        .run(&mut evm)
        .expect("runs");
    figures(&result)
}

/// Every case, named.
fn cases() -> Vec<(String, String)> {
    use InstructionResult::{OutOfGas, Revert, Stop};
    let mut shapes = Vec::new();
    for status in [Stop, Revert, OutOfGas] {
        for left in [Left::All, Left::Half] {
            for reservoir in [0, 1] {
                shapes.push(Shape::Answer(status, left, reservoir));
            }
        }
    }
    for left in [Left::All, Left::Half] {
        for reservoir in [0, 1] {
            shapes.push(Shape::ReplaceGas(left, reservoir));
        }
    }
    shapes.push(Shape::RebuildOutcome);
    shapes.push(Shape::RewriteStatus(Revert));
    shapes.push(Shape::RewriteStatus(OutOfGas));
    shapes.push(Shape::RewriteAndReplace(OutOfGas, 1));
    shapes.push(Shape::RewriteAndReplace(Revert, 1));

    let mut cases = Vec::new();
    for target in [
        Target::Root,
        Target::Child,
        Target::RootCreation,
        Target::ChildCreation,
    ] {
        cases.push((format!("{target:?} plain"), plain(target)));
        for shape in &shapes {
            cases.push((
                format!("{target:?} inspected {shape:?}"),
                inspected(target, *shape),
            ));
            if let Shape::Answer(status, left, reservoir) = *shape {
                cases.push((
                    format!("{target:?} frame_init {shape:?}"),
                    answered(target, (status, left, reservoir)),
                ));
            }
        }
    }
    cases
}

/// The figures revm computed for each case before the lifecycle existed (`b370b538`).
const EXPECTED: &[(&str, &str)] = &[
    (
        "Root plain",
        "Success(Stop) spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Stop, All, 0)",
        "Success(Stop) spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Stop, All, 0)",
        "Success(Stop) spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Stop, All, 1)",
        "Success(Stop) spent 204599 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Stop, All, 1)",
        "Success(Stop) spent 204599 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Stop, Half, 0)",
        "Success(Stop) spent 602300 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Stop, Half, 0)",
        "Success(Stop) spent 602300 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Stop, Half, 1)",
        "Success(Stop) spent 602299 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Stop, Half, 1)",
        "Success(Stop) spent 602299 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Revert, All, 0)",
        "Revert spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Revert, All, 0)",
        "Revert spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Revert, All, 1)",
        "Revert spent 204599 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Revert, All, 1)",
        "Revert spent 204599 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Revert, Half, 0)",
        "Revert spent 602300 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Revert, Half, 0)",
        "Revert spent 602300 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(Revert, Half, 1)",
        "Revert spent 602299 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(Revert, Half, 1)",
        "Revert spent 602299 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(OutOfGas, All, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(OutOfGas, All, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(OutOfGas, All, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(OutOfGas, All, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(OutOfGas, Half, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(OutOfGas, Half, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected Answer(OutOfGas, Half, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root frame_init Answer(OutOfGas, Half, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected ReplaceGas(All, 0)",
        "Success(Stop) spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected ReplaceGas(All, 1)",
        "Success(Stop) spent 204599 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected ReplaceGas(Half, 0)",
        "Success(Stop) spent 602300 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected ReplaceGas(Half, 1)",
        "Success(Stop) spent 602299 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected RebuildOutcome",
        "Success(Stop) spent 204600 state 183600 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected RewriteStatus(Revert)",
        "Revert spent 21000 state 0 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected RewriteStatus(OutOfGas)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 0 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected RewriteAndReplace(OutOfGas, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 0 refunded 0 floor 21000 output 0x",
    ),
    (
        "Root inspected RewriteAndReplace(Revert, 1)",
        "Revert spent 20999 state 0 refunded 0 floor 21000 output 0x",
    ),
    (
        "Child plain",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Stop, All, 0)",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Stop, All, 0)",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Stop, All, 1)",
        "Success(Stop) spent 210616 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Stop, All, 1)",
        "Success(Stop) spent 210616 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Stop, Half, 0)",
        "Success(Stop) spent 244535 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Stop, Half, 0)",
        "Success(Stop) spent 244535 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Stop, Half, 1)",
        "Success(Stop) spent 244534 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Stop, Half, 1)",
        "Success(Stop) spent 244534 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Revert, All, 0)",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Revert, All, 0)",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Revert, All, 1)",
        "Success(Stop) spent 210616 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Revert, All, 1)",
        "Success(Stop) spent 210616 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Revert, Half, 0)",
        "Success(Stop) spent 244535 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Revert, Half, 0)",
        "Success(Stop) spent 244535 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(Revert, Half, 1)",
        "Success(Stop) spent 244534 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(Revert, Half, 1)",
        "Success(Stop) spent 244534 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(OutOfGas, All, 0)",
        "Success(Stop) spent 278453 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(OutOfGas, All, 0)",
        "Success(Stop) spent 278453 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(OutOfGas, All, 1)",
        "Success(Stop) spent 278452 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(OutOfGas, All, 1)",
        "Success(Stop) spent 278452 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(OutOfGas, Half, 0)",
        "Success(Stop) spent 278453 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(OutOfGas, Half, 0)",
        "Success(Stop) spent 278453 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected Answer(OutOfGas, Half, 1)",
        "Success(Stop) spent 278452 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child frame_init Answer(OutOfGas, Half, 1)",
        "Success(Stop) spent 278452 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected ReplaceGas(All, 0)",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected ReplaceGas(All, 1)",
        "Success(Stop) spent 210616 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected ReplaceGas(Half, 0)",
        "Success(Stop) spent 244535 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected ReplaceGas(Half, 1)",
        "Success(Stop) spent 244534 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected RebuildOutcome",
        "Success(Stop) spent 210617 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected RewriteStatus(Revert)",
        "Success(Stop) spent 27017 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected RewriteStatus(OutOfGas)",
        "Success(Stop) spent 94853 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected RewriteAndReplace(OutOfGas, 1)",
        "Success(Stop) spent 94852 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "Child inspected RewriteAndReplace(Revert, 1)",
        "Success(Stop) spent 27016 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "RootCreation plain",
        "Success(Return) spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Stop, All, 0)",
        "Success(Stop) spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Stop, All, 0)",
        "Success(Stop) spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Stop, All, 1)",
        "Success(Stop) spent 207599 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Stop, All, 1)",
        "Success(Stop) spent 207599 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Stop, Half, 0)",
        "Success(Stop) spent 603800 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Stop, Half, 0)",
        "Success(Stop) spent 603800 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Stop, Half, 1)",
        "Success(Stop) spent 603799 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Stop, Half, 1)",
        "Success(Stop) spent 603799 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Revert, All, 0)",
        "Revert spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Revert, All, 0)",
        "Revert spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Revert, All, 1)",
        "Revert spent 207599 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Revert, All, 1)",
        "Revert spent 207599 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Revert, Half, 0)",
        "Revert spent 603800 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Revert, Half, 0)",
        "Revert spent 603800 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(Revert, Half, 1)",
        "Revert spent 603799 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(Revert, Half, 1)",
        "Revert spent 603799 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(OutOfGas, All, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(OutOfGas, All, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(OutOfGas, All, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(OutOfGas, All, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(OutOfGas, Half, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(OutOfGas, Half, 0)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected Answer(OutOfGas, Half, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation frame_init Answer(OutOfGas, Half, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected ReplaceGas(All, 0)",
        "Success(Return) spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected ReplaceGas(All, 1)",
        "Success(Return) spent 207599 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected ReplaceGas(Half, 0)",
        "Success(Return) spent 603800 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected ReplaceGas(Half, 1)",
        "Success(Return) spent 603799 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected RebuildOutcome",
        "Success(Return) spent 207600 state 183600 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected RewriteStatus(Revert)",
        "Revert spent 24000 state 0 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected RewriteStatus(OutOfGas)",
        "Halt(OutOfGas(Basic)) spent 1000000 state 0 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected RewriteAndReplace(OutOfGas, 1)",
        "Halt(OutOfGas(Basic)) spent 999999 state 0 refunded 0 floor 24000 output 0x",
    ),
    (
        "RootCreation inspected RewriteAndReplace(Revert, 1)",
        "Revert spent 23999 state 0 refunded 0 floor 24000 output 0x",
    ),
    (
        "ChildCreation plain",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Stop, All, 0)",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Stop, All, 0)",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Stop, All, 1)",
        "Success(Stop) spent 210605 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Stop, All, 1)",
        "Success(Stop) spent 210605 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Stop, Half, 0)",
        "Success(Stop) spent 599136 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Stop, Half, 0)",
        "Success(Stop) spent 599136 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Stop, Half, 1)",
        "Success(Stop) spent 599135 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Stop, Half, 1)",
        "Success(Stop) spent 599135 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Revert, All, 0)",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Revert, All, 0)",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Revert, All, 1)",
        "Success(Stop) spent 210605 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Revert, All, 1)",
        "Success(Stop) spent 210605 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Revert, Half, 0)",
        "Success(Stop) spent 599136 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Revert, Half, 0)",
        "Success(Stop) spent 599136 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(Revert, Half, 1)",
        "Success(Stop) spent 599135 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(Revert, Half, 1)",
        "Success(Stop) spent 599135 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(OutOfGas, All, 0)",
        "Success(Stop) spent 987666 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(OutOfGas, All, 0)",
        "Success(Stop) spent 987666 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(OutOfGas, All, 1)",
        "Success(Stop) spent 987665 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(OutOfGas, All, 1)",
        "Success(Stop) spent 987665 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(OutOfGas, Half, 0)",
        "Success(Stop) spent 987666 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(OutOfGas, Half, 0)",
        "Success(Stop) spent 987666 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected Answer(OutOfGas, Half, 1)",
        "Success(Stop) spent 987665 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation frame_init Answer(OutOfGas, Half, 1)",
        "Success(Stop) spent 987665 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected ReplaceGas(All, 0)",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected ReplaceGas(All, 1)",
        "Success(Stop) spent 210605 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected ReplaceGas(Half, 0)",
        "Success(Stop) spent 599136 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected ReplaceGas(Half, 1)",
        "Success(Stop) spent 599135 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected RebuildOutcome",
        "Success(Stop) spent 210606 state 183600 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected RewriteStatus(Revert)",
        "Success(Stop) spent 27006 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected RewriteStatus(OutOfGas)",
        "Success(Stop) spent 804066 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected RewriteAndReplace(OutOfGas, 1)",
        "Success(Stop) spent 804065 state 0 refunded 0 floor 15000 output 0x",
    ),
    (
        "ChildCreation inspected RewriteAndReplace(Revert, 1)",
        "Success(Stop) spent 27005 state 0 refunded 0 floor 15000 output 0x",
    ),
];

#[test]
fn test_default_hooks_change_no_figure_the_settlement_computes() {
    let actual = cases();
    let table: String = actual
        .iter()
        .map(|(name, figures)| format!("    ({name:?}, {figures:?}),\n"))
        .collect();
    let expected: Vec<(String, String)> = EXPECTED
        .iter()
        .map(|(name, figures)| (name.to_string(), figures.to_string()))
        .collect();
    assert!(
        actual == expected,
        "the figures moved; this revision's table:\n{table}"
    );
}

/// The reviewer's case: the transaction's frame answered with an out-of-gas outcome whose tracker
/// hands back a reservoir of 1, which unwinds 1 of the transaction's spill into regular gas. The
/// halt keeps that gas, as revm's settlement left it, whether an inspector or the consumer's
/// `frame_init` answers.
#[test]
fn test_a_halted_answer_to_the_transactions_frame_keeps_the_gas_its_settlement_returned() {
    let answer = (InstructionResult::OutOfGas, Left::All, 1);
    for figures in [
        inspected(Target::Root, Shape::Answer(answer.0, answer.1, answer.2)),
        answered(Target::Root, answer),
    ] {
        assert!(
            figures.starts_with("Halt(OutOfGas(Basic)) spent 999999 "),
            "{figures}"
        );
    }
}
