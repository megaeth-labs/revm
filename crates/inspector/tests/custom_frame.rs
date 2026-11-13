//! The inspected run of a consumer's own frame type, which wraps revm's [`EthFrame`] and gives the
//! inspector that frame ([`InspectorFrame::eth_frame`]) only while it has not finished. Nothing
//! asks the accessor to ignore the frame's state, so the run looks the frame up for the inspector's
//! end callbacks before it marks the frame finished, as upstream's does: the inspector sees each
//! frame's `call_end` and `frame_end`.
//!
//! [`PARENT`] calls [`CHILD`], and both stop. The consumer sets no action and keeps
//! [`EvmTr::frame_run_result`]'s default; its EVM runs its frames as revm's own does.

use context::{
    result::{EVMError, ExecutionResult, HaltReason},
    Context, ContextSetters, Evm, FrameStack, OutFrame, TxEnv,
};
use database::{CacheDB, EmptyDB, BENCH_CALLER};
use handler::{
    evm::{ContextDbError, FrameInitResult, FrameTr},
    instructions::{EthInstructions, InstructionProvider},
    lifecycle, EthFrame, EthPrecompiles, EvmTr, FrameInitOrResult, FrameResult, Handler,
    ItemOrResult, MainContext, MainnetContext,
};
use interpreter::{
    interpreter::EthInterpreter, interpreter_action::FrameInit, CallInputs, CallOutcome, FrameInput,
};
use primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256};
use revm_inspector::{Inspector, InspectorEvmTr, InspectorFrame, InspectorHandler};
use state::{
    bytecode::opcode::{CALL, GAS, PUSH0, PUSH20, STOP},
    AccountInfo, Bytecode,
};
use std::convert::Infallible;

type Db = CacheDB<EmptyDB>;
type Ctx = MainnetContext<Db>;
type Instructions = EthInstructions<EthInterpreter, Ctx>;
type Inner = Evm<Ctx, Recorder, Instructions, EthPrecompiles, HiddenOnceFinished>;
type Error = EVMError<Infallible>;

const SPEC: SpecId = SpecId::OSAKA;
const GAS_LIMIT: u64 = 1_000_000;

/// Calls [`CHILD`] with all its gas, and stops.
const PARENT: Address = address!("0x00000000000000000000000000000000000c0de0");
/// Stops.
const CHILD: Address = address!("0x00000000000000000000000000000000000c0de1");

/// revm's frame, which the inspector is given only while it has not finished.
#[derive(Debug, Default)]
struct HiddenOnceFinished(EthFrame);

impl FrameTr for HiddenOnceFinished {
    type FrameInit = FrameInit;
    type FrameResult = FrameResult;
}

impl InspectorFrame for HiddenOnceFinished {
    type IT = EthInterpreter;

    fn eth_frame(&mut self) -> Option<&mut EthFrame> {
        if self.0.is_finished() {
            None
        } else {
            Some(&mut self.0)
        }
    }
}

/// The frames' starts and ends, one line per callback, by the account the frame runs as.
#[derive(Debug, Default)]
struct Recorder {
    events: Vec<String>,
}

const fn name(address: Address) -> &'static str {
    match address {
        PARENT => "PARENT",
        CHILD => "CHILD",
        _ => "other",
    }
}

impl Inspector<Ctx> for Recorder {
    fn call(&mut self, _: &mut Ctx, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let account = name(inputs.target_address);
        self.events.push(format!("call {account}"));
        None
    }

    fn call_end(&mut self, _: &mut Ctx, inputs: &CallInputs, outcome: &mut CallOutcome) {
        let (account, result) = (name(inputs.target_address), outcome.result.result);
        self.events.push(format!("call_end {account} {result:?}"));
    }

    fn frame_end(&mut self, _: &mut Ctx, input: &FrameInput, result: &mut FrameResult) {
        let account = match input {
            FrameInput::Call(inputs) => name(inputs.target_address),
            _ => "other",
        };
        let result = result.instruction_result();
        self.events.push(format!("frame_end {account} {result:?}"));
    }
}

/// revm's EVM on [`HiddenOnceFinished`]: its frame start, run and return are revm's own, on the
/// [`EthFrame`] inside.
struct Consumer {
    evm: Inner,
}

impl EvmTr for Consumer {
    type Context = Ctx;
    type Instructions = Instructions;
    type Precompiles = EthPrecompiles;
    type Frame = HiddenOnceFinished;

    fn all(
        &self,
    ) -> (
        &Ctx,
        &Instructions,
        &EthPrecompiles,
        &FrameStack<HiddenOnceFinished>,
    ) {
        let evm = &self.evm;
        (
            &evm.ctx,
            &evm.instruction,
            &evm.precompiles,
            &evm.frame_stack,
        )
    }

    fn all_mut(
        &mut self,
    ) -> (
        &mut Ctx,
        &mut Instructions,
        &mut EthPrecompiles,
        &mut FrameStack<HiddenOnceFinished>,
    ) {
        let evm = &mut self.evm;
        (
            &mut evm.ctx,
            &mut evm.instruction,
            &mut evm.precompiles,
            &mut evm.frame_stack,
        )
    }

    fn frame_init(
        &mut self,
        frame_input: FrameInit,
    ) -> Result<FrameInitResult<'_, HiddenOnceFinished>, ContextDbError<Ctx>> {
        let Evm {
            ctx,
            precompiles,
            frame_stack,
            ..
        } = &mut self.evm;
        let is_first_init = frame_stack.index().is_none();
        let mut slot = if is_first_init {
            frame_stack.start_init()
        } else {
            frame_stack.get_next()
        };
        let frame = &mut slot.get(HiddenOnceFinished::default).0;
        let out = OutFrame::new_init(frame);
        if let ItemOrResult::Result(result) =
            EthFrame::init_with_context(out, ctx, precompiles, frame_input)?
        {
            return Ok(ItemOrResult::Result(result));
        }
        let token = slot.consume();
        // SAFETY: the token is the slot's this stack just gave, which `slot.get` initialized.
        unsafe {
            if is_first_init {
                frame_stack.end_init(token);
            } else {
                frame_stack.push(token);
            }
        }
        Ok(ItemOrResult::Item(frame_stack.get()))
    }

    fn frame_run(&mut self) -> Result<FrameInitOrResult<HiddenOnceFinished>, ContextDbError<Ctx>> {
        let Evm {
            ctx,
            instruction,
            frame_stack,
            ..
        } = &mut self.evm;
        let frame = &mut frame_stack.get().0;
        let action = frame.interpreter.run_plain(
            instruction.instruction_table(),
            instruction.gas_table(),
            ctx,
        );
        let result = frame.process_next_action(ctx, action);
        if let Ok(ItemOrResult::Result(_)) = &result {
            frame.set_finished(true);
        }
        // An EVM that runs its frames itself opens the start of the child init it consumed.
        if let Ok(ItemOrResult::Item(init)) = &result {
            lifecycle::open_frame_start(self, init)?;
        }
        result
    }

    fn frame_return_result(
        &mut self,
        result: FrameResult,
    ) -> Result<Option<FrameResult>, ContextDbError<Ctx>> {
        let Evm {
            ctx, frame_stack, ..
        } = &mut self.evm;
        if frame_stack.get().0.is_finished() {
            frame_stack.pop();
        }
        if frame_stack.index().is_none() {
            return Ok(Some(result));
        }
        // An EVM that returns results itself settles the caller's open start after the return.
        let caller = &mut frame_stack.get().0;
        let returning = lifecycle::Returning::new(&result, caller);
        caller.return_result::<_, ContextDbError<Ctx>>(ctx, result)?;
        lifecycle::close_frame_start(ctx, frame_stack, returning, |frame| &mut frame.0)?;
        Ok(None)
    }
}

impl InspectorEvmTr for Consumer {
    type Inspector = Recorder;

    fn all_inspector(
        &self,
    ) -> (
        &Ctx,
        &Instructions,
        &EthPrecompiles,
        &FrameStack<HiddenOnceFinished>,
        &Recorder,
    ) {
        let evm = &self.evm;
        (
            &evm.ctx,
            &evm.instruction,
            &evm.precompiles,
            &evm.frame_stack,
            &evm.inspector,
        )
    }

    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Ctx,
        &mut Instructions,
        &mut EthPrecompiles,
        &mut FrameStack<HiddenOnceFinished>,
        &mut Recorder,
    ) {
        let evm = &mut self.evm;
        (
            &mut evm.ctx,
            &mut evm.instruction,
            &mut evm.precompiles,
            &mut evm.frame_stack,
            &mut evm.inspector,
        )
    }
}

/// The mainnet handler, on [`Consumer`].
struct Mainnet;

impl Handler for Mainnet {
    type Evm = Consumer;
    type Error = Error;
    type HaltReason = HaltReason;
}

impl InspectorHandler for Mainnet {
    type IT = EthInterpreter;
}

/// A [`Consumer`] holding [`PARENT`] and [`CHILD`], with a transaction to [`PARENT`].
fn consumer() -> Consumer {
    let mut db = Db::default();
    db.insert_account_info(
        BENCH_CALLER,
        AccountInfo::from_balance(U256::from(10u128.pow(21))),
    );
    let mut parent = vec![PUSH0, PUSH0, PUSH0, PUSH0, PUSH0, PUSH20];
    parent.extend_from_slice(CHILD.as_slice());
    parent.extend_from_slice(&[GAS, CALL, STOP]);
    let codes = [
        (PARENT, parent.into()),
        (CHILD, Bytes::from_static(&[STOP])),
    ];
    for (address, code) in codes {
        let code = Bytecode::new_raw(code);
        db.insert_account_info(
            address,
            AccountInfo::new(U256::ZERO, 1, code.hash_slow(), code),
        );
    }
    let mut ctx = Context::mainnet()
        .with_db(db)
        .modify_cfg_chained(|cfg| cfg.set_spec_and_mainnet_gas_params(SPEC));
    ctx.set_tx(
        TxEnv::builder_for_bench()
            .kind(TxKind::Call(PARENT))
            .gas_price(0)
            .gas_limit(GAS_LIMIT)
            .build_fill(),
    );
    Consumer {
        evm: Evm {
            ctx,
            inspector: Recorder::default(),
            instruction: Instructions::new_mainnet_with_spec(SPEC),
            precompiles: EthPrecompiles::new(SPEC),
            frame_stack: FrameStack::new(),
        },
    }
}

/// The inspector sees each frame's end, though the frame's accessor gives it no finished frame,
/// and the transaction's result is the plain run's.
#[test]
fn test_a_frame_hidden_once_finished_gets_its_end_callbacks() {
    let mut inspected = consumer();
    let result = Mainnet
        .inspect_run(&mut inspected)
        .expect("the transaction runs");

    assert_eq!(
        inspected.evm.inspector.events,
        [
            "call PARENT",
            "call CHILD",
            "call_end CHILD Stop",
            "frame_end CHILD Stop",
            "call_end PARENT Stop",
            "frame_end PARENT Stop",
        ]
    );
    assert!(
        matches!(result, ExecutionResult::Success { .. }),
        "{result:?}"
    );
    let plain = Mainnet.run(&mut consumer()).expect("the transaction runs");
    assert_eq!(result, plain);
}
