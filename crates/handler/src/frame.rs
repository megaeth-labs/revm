use crate::{
    evm::FrameTr, item_or_result::FrameInitOrResult, precompile_provider::PrecompileProvider,
    CallFrame, CreateFrame, FrameData, FrameResult, ItemOrResult,
};
use context::result::FromStringError;
use context_interface::{
    cfg::{GasId, StateGasCharge, StateGasSite},
    context::{take_error, ContextError},
    journaled_state::{account::JournaledAccountTr, JournalCheckpoint, JournalTr},
    local::{FrameToken, OutFrame},
    Cfg, ContextTr, Database,
};
use core::cmp::min;
use derive_where::derive_where;
use interpreter::{
    interpreter::{EthInterpreter, ExtBytecode},
    interpreter_action::FrameInit,
    interpreter_types::ReturnData,
    CallInput, CallInputs, CallOutcome, CallValue, CreateInputs, CreateOutcome, CreateScheme,
    FrameInput, Gas, GasTracker, InputsImpl, InstructionResult, Interpreter, InterpreterAction,
    InterpreterResult, InterpreterTypes, SharedMemory, StartCharge,
};
use primitives::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, HOMESTEAD, LONDON, SPURIOUS_DRAGON},
    Address, Bytes, U256,
};
use state::Bytecode;
use std::{borrow::ToOwned, boxed::Box, string::ToString, vec::Vec};

mod code_deposit;

/// Frame implementation for Ethereum.
#[derive_where(Clone, Debug; IW,
    <IW as InterpreterTypes>::Stack,
    <IW as InterpreterTypes>::Memory,
    <IW as InterpreterTypes>::Bytecode,
    <IW as InterpreterTypes>::ReturnData,
    <IW as InterpreterTypes>::Input,
    <IW as InterpreterTypes>::RuntimeFlag,
    <IW as InterpreterTypes>::Extend,
)]
pub struct EthFrame<IW: InterpreterTypes = EthInterpreter> {
    /// Frame-specific data (Call, Create, or EOFCreate).
    pub data: FrameData,
    /// Input data for the frame.
    pub input: FrameInput,
    /// Current call depth in the execution stack.
    pub depth: usize,
    /// Journal checkpoint for state reversion.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter instance for executing bytecode.
    pub interpreter: Interpreter<IW>,
    /// Whether the frame has been finished its execution.
    /// Frame is considered finished if it has been called and returned a result.
    pub is_finished: bool,
}

impl<IT: InterpreterTypes> FrameTr for EthFrame<IT> {
    type FrameResult = FrameResult;
    type FrameInit = FrameInit;
}

impl Default for EthFrame<EthInterpreter> {
    fn default() -> Self {
        Self::do_default(Interpreter::default())
    }
}

impl EthFrame<EthInterpreter> {
    /// Creates an new invalid [`EthFrame`].
    pub fn invalid() -> Self {
        Self::do_default(Interpreter::invalid())
    }

    fn do_default(interpreter: Interpreter<EthInterpreter>) -> Self {
        Self {
            data: FrameData::Call(CallFrame {
                return_memory_range: 0..0,
            }),
            input: FrameInput::Empty,
            depth: 0,
            checkpoint: JournalCheckpoint::default(),
            interpreter,
            is_finished: false,
        }
    }

    /// Returns true if the frame has finished execution.
    pub const fn is_finished(&self) -> bool {
        self.is_finished
    }

    /// Sets the finished state of the frame.
    pub const fn set_finished(&mut self, finished: bool) {
        self.is_finished = finished;
    }
}

/// Type alias for database errors from a context.
pub type ContextTrDbError<CTX> = <<CTX as ContextTr>::Db as Database>::Error;

impl EthFrame<EthInterpreter> {
    /// Clear and initialize a frame.
    #[expect(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn clear(
        &mut self,
        data: FrameData,
        input: FrameInput,
        depth: usize,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        inputs: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
        reservoir_remaining_gas: u64,
        checkpoint: JournalCheckpoint,
    ) {
        let Self {
            data: data_ref,
            input: input_ref,
            depth: depth_ref,
            interpreter,
            checkpoint: checkpoint_ref,
            is_finished: is_finished_ref,
        } = self;
        *data_ref = data;
        *input_ref = input;
        *depth_ref = depth;
        *is_finished_ref = false;
        interpreter.clear(
            memory,
            bytecode,
            inputs,
            is_static,
            spec_id,
            gas_limit,
            reservoir_remaining_gas,
        );
        *checkpoint_ref = checkpoint;
    }

    /// Make call frame
    #[inline]
    pub fn make_call_frame<
        CTX: ContextTr,
        PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        mut this: OutFrame<'_, Self>,
        ctx: &mut CTX,
        precompiles: &mut PRECOMPILES,
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CallInputs>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR> {
        let reservoir_remaining_gas = inputs.reservoir;
        let charged_new_account_state_gas = inputs.charged_new_account_state_gas;
        let charged_state_gas_address = inputs.target_address;
        let charged_start = inputs.charged_start;
        // The child's pools are the shares its caller forwarded, and its history allowance what
        // the call instruction gave it; both ride on the frame's gas, as the reservoir does,
        // whether the handler runs the frame or answers the start without one.
        let pool_shares = inputs.pool_shares;
        let history_allowance = inputs.history_allowance;
        let mut gas =
            Gas::new_with_regular_gas_and_reservoir(inputs.gas_limit, reservoir_remaining_gas);
        gas.tracker_mut().set_pools_remaining(pool_shares);
        gas.set_history_allowance(history_allowance);

        let return_result = |instruction_result: InstructionResult| {
            Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                result: InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                memory_offset: inputs.return_memory_offset.clone(),
                was_precompile_called: false,
                precompile_call_logs: Vec::new(),
                charged_new_account_state_gas,
                charged_state_gas_address,
                charged_start,
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_result(InstructionResult::CallTooDeep);
        }

        // Create subroutine checkpoint
        let checkpoint = ctx.journal_mut().checkpoint();

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if let CallValue::Transfer(value) = inputs.value {
            // Transfer value from caller to called account
            // Target will get touched even if balance transferred is zero.
            if let Some(i) =
                ctx.journal_mut()
                    .transfer_loaded(inputs.caller, inputs.target_address, value)
            {
                ctx.journal_mut().checkpoint_revert(checkpoint);
                return return_result(i.into());
            }
        }

        let interpreter_input = InputsImpl {
            target_address: inputs.target_address,
            caller_address: inputs.caller,
            bytecode_address: Some(inputs.bytecode_address),
            input: inputs.input.clone(),
            call_value: inputs.value.get(),
            depth,
        };
        let is_static = inputs.is_static;
        let gas_limit = inputs.gas_limit;

        if let Some(mut result) = precompiles.run(ctx, &inputs).map_err(ERROR::from_string)? {
            // The provider built the answer's gas from the inputs' regular gas and reservoir. The
            // answer takes the caller's shares as the pools it started with, less what the
            // provider drew on them (a draw past a share is held to it, with the pool's
            // crossing), and the allowance the input carries, so the caller's settlement gets
            // the shares back from the answer as it would from a frame.
            result.gas.tracker_mut().adopt_pool_shares(pool_shares);
            result.gas.set_history_allowance(history_allowance);
            let mut logs = Vec::new();
            if result.result.is_ok() {
                // Preserve the reservoir on the result gas so it can be reimbursed.
                // Precompiles don't use reservoir gas, but the first frame carries it.
                ctx.journal_mut().checkpoint_commit();
            } else {
                // clone logs that precompile created, only possible with custom precompiles.
                // checkpoint.log_i will be always correct.
                logs = ctx.journal_mut().logs()[checkpoint.log_i..].to_vec();
                ctx.journal_mut().checkpoint_revert(checkpoint);
            }
            return Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                result,
                memory_offset: inputs.return_memory_offset.clone(),
                was_precompile_called: true,
                precompile_call_logs: logs,
                charged_new_account_state_gas,
                charged_state_gas_address,
                charged_start,
            })));
        }

        // Get bytecode and hash - either from known_bytecode or load from account
        let (bytecode_hash, bytecode) = inputs.known_bytecode.clone();

        // Returns success if bytecode is empty.
        if bytecode.is_empty() {
            ctx.journal_mut().checkpoint_commit();
            return return_result(InstructionResult::Stop);
        }

        // Create interpreter and executes call and push new CallStackFrame.
        let frame = this.get(EthFrame::invalid);
        frame.clear(
            FrameData::Call(CallFrame {
                return_memory_range: inputs.return_memory_offset.clone(),
            }),
            FrameInput::Call(inputs),
            depth,
            memory,
            ExtBytecode::new_with_hash(bytecode, bytecode_hash),
            interpreter_input,
            is_static,
            ctx.cfg().spec().into(),
            gas_limit,
            reservoir_remaining_gas,
            checkpoint,
        );
        frame
            .interpreter
            .gas
            .tracker_mut()
            .set_pools_remaining(pool_shares);
        frame
            .interpreter
            .gas
            .set_history_allowance(history_allowance);

        Ok(ItemOrResult::Item(this.consume()))
    }

    /// Make create frame.
    #[inline]
    pub fn make_create_frame<
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        mut this: OutFrame<'_, Self>,
        context: &mut CTX,
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CreateInputs>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR> {
        let reservoir_remaining_gas = inputs.reservoir();
        let spec = context.cfg().spec().into();
        // EIP-8037 refund for the CREATE opcode's upfront `create_state_gas` is
        // applied uniformly in `return_result` when the create fails (revert,
        // halt, or early-fail with `address == None`), so early-fail results
        // only carry the reservoir they inherited from the parent.
        let charged_create_state_gas = inputs.charged_create_state_gas();
        // Recorded by whoever made the charge — the CREATE opcode or the EIP-2780 runtime phase
        // — because the created address is not derivable here on the early-fail paths.
        let charged_state_gas_address = inputs.charged_state_gas_address();
        let charged_start = inputs.charged_start();
        // The creation's pools are the shares its caller forwarded; they ride on the frame's
        // gas, as the reservoir does, on an early failure too, so the caller's settlement gets
        // them back.
        let pool_shares = inputs.pool_shares();
        let return_error = |e| {
            let mut gas = Gas::new_with_regular_gas_and_reservoir(
                inputs.gas_limit(),
                reservoir_remaining_gas,
            );
            gas.tracker_mut().set_pools_remaining(pool_shares);
            Ok(ItemOrResult::Result(FrameResult::Create(CreateOutcome {
                result: InterpreterResult {
                    result: e,
                    gas,
                    output: Bytes::new(),
                },
                address: None,
                charged_create_state_gas,
                charged_state_gas_address,
                charged_start,
            })))
        };

        // Check depth
        if depth > CALL_STACK_LIMIT as usize {
            return return_error(InstructionResult::CallTooDeep);
        }

        // Fetch balance of caller.
        let journal = context.journal_mut();
        let mut caller_info = journal.load_account_mut(inputs.caller())?;

        // Check if caller has enough balance to send to the created contract.
        // decrement of balance is done in the create_account_checkpoint.
        if *caller_info.balance() < inputs.value() {
            return return_error(InstructionResult::OutOfFunds);
        }

        // Increase nonce of caller and check if it overflows
        let old_nonce = caller_info.nonce();
        if !caller_info.bump_nonce() {
            return return_error(InstructionResult::Return);
        };

        // Create address — uses OnceCell cache so that if an inspector already called
        // `created_address`, the expensive keccak256 is not recomputed.
        let created_address = inputs.created_address(old_nonce);
        let init_code_hash = matches!(inputs.scheme(), CreateScheme::Create2 { .. })
            .then(|| inputs.init_code_hash());

        drop(caller_info); // Drop caller info to avoid borrow checker issues.

        // warm load account.
        journal.load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let checkpoint = match context.journal_mut().create_account_checkpoint(
            inputs.caller(),
            created_address,
            inputs.value(),
            spec,
        ) {
            Ok(checkpoint) => checkpoint,
            Err(e) => return return_error(e.into()),
        };

        let bytecode = ExtBytecode::new_with_optional_hash(
            Bytecode::new_legacy(inputs.init_code().clone()),
            init_code_hash,
        );

        let interpreter_input = InputsImpl {
            target_address: created_address,
            caller_address: inputs.caller(),
            bytecode_address: None,
            input: CallInput::Bytes(Bytes::new()),
            call_value: inputs.value(),
            depth,
        };
        let gas_limit = inputs.gas_limit();

        let frame = this.get(EthFrame::invalid);
        frame.clear(
            FrameData::Create(CreateFrame { created_address }),
            FrameInput::Create(inputs),
            depth,
            memory,
            bytecode,
            interpreter_input,
            false,
            spec,
            gas_limit,
            reservoir_remaining_gas,
            checkpoint,
        );
        frame
            .interpreter
            .gas
            .tracker_mut()
            .set_pools_remaining(pool_shares);

        Ok(ItemOrResult::Item(this.consume()))
    }

    /// Initializes a frame with the given context and precompiles.
    pub fn init_with_context<
        CTX: ContextTr,
        PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
    >(
        this: OutFrame<'_, Self>,
        ctx: &mut CTX,
        precompiles: &mut PRECOMPILES,
        frame_init: FrameInit,
    ) -> Result<
        ItemOrResult<FrameToken, FrameResult>,
        ContextError<<<CTX as ContextTr>::Db as Database>::Error>,
    > {
        // TODO cleanup inner make functions
        let FrameInit {
            depth,
            memory,
            frame_input,
        } = frame_init;

        match frame_input {
            FrameInput::Call(inputs) => {
                Self::make_call_frame(this, ctx, precompiles, depth, memory, inputs)
            }
            FrameInput::Create(inputs) => Self::make_create_frame(this, ctx, depth, memory, inputs),
            FrameInput::Empty => unreachable!(),
        }
    }
}

impl EthFrame<EthInterpreter> {
    /// Processes the next interpreter action, either creating a new frame or returning a result.
    pub fn process_next_action<
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        &mut self,
        context: &mut CTX,
        next_action: InterpreterAction,
    ) -> Result<FrameInitOrResult<Self>, ERROR> {
        // Run interpreter

        let mut interpreter_result = match next_action {
            InterpreterAction::NewFrame(frame_input) => {
                let depth = self.depth + 1;
                return Ok(ItemOrResult::Item(FrameInit {
                    frame_input,
                    depth,
                    memory: self.interpreter.memory.new_child_context(),
                }));
            }
            InterpreterAction::Return(result) => result,
        };

        // Handle return from frame
        let result = match &self.data {
            FrameData::Call(frame) => {
                // return_call
                // Revert changes or not.
                if interpreter_result.result.is_ok() {
                    context.journal_mut().checkpoint_commit();
                } else {
                    context.journal_mut().checkpoint_revert(self.checkpoint);
                }
                // Propagate EIP-8037 new-account state-gas flag from the frame
                // input so the parent can refund the upfront charge if the call
                // ends in revert/halt.
                // The start charge rides along for the same refund.
                let (charged_new_account_state_gas, charged_state_gas_address, charged_start) =
                    match &self.input {
                        FrameInput::Call(inputs) => (
                            inputs.charged_new_account_state_gas,
                            inputs.target_address,
                            inputs.charged_start,
                        ),
                        _ => (false, Address::ZERO, StartCharge::default()),
                    };
                let mut outcome =
                    CallOutcome::new(interpreter_result, frame.return_memory_range.clone());
                outcome.charged_new_account_state_gas = charged_new_account_state_gas;
                outcome.charged_state_gas_address = charged_state_gas_address;
                outcome.charged_start = charged_start;
                ItemOrResult::Result(FrameResult::Call(outcome))
            }
            FrameData::Create(frame) => {
                return_create(
                    context,
                    self.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                );

                let mut create_outcome =
                    CreateOutcome::new(interpreter_result, Some(frame.created_address));
                // The charged address comes from the inputs, not `frame.created_address`: the
                // inputs carry the address the charge site recorded, which is the one the
                // refund must use. An uncharged creation leaves it zero, so the field is
                // meaningful only when `charged_create_state_gas` is set.
                let (charged_create_state_gas, charged_state_gas_address, charged_start) =
                    match &self.input {
                        FrameInput::Create(inputs) => (
                            inputs.charged_create_state_gas(),
                            inputs.charged_state_gas_address(),
                            inputs.charged_start(),
                        ),
                        _ => (false, Address::ZERO, StartCharge::default()),
                    };
                create_outcome.charged_create_state_gas = charged_create_state_gas;
                create_outcome.charged_state_gas_address = charged_state_gas_address;
                create_outcome.charged_start = charged_start;
                ItemOrResult::Result(FrameResult::Create(create_outcome))
            }
        };

        Ok(result)
    }

    /// Processes a frame result and updates the interpreter state accordingly.
    pub fn return_result<CTX: ContextTr, ERROR: From<ContextTrDbError<CTX>> + FromStringError>(
        &mut self,
        ctx: &mut CTX,
        result: FrameResult,
    ) -> Result<(), ERROR> {
        self.interpreter.memory.free_child_context();
        take_error::<ERROR, _>(ctx.error())?;

        // EIP-8037: the CALL/CREATE opcode charged the new-account or
        // create state gas upfront on this (parent) frame's tracker. When the
        // child does not create the account leaf it paid for, the charge is
        // refunded below via `refill_reservoir` (matching 0→x→0 storage
        // restoration) — the child rollback in `handle_reservoir_remaining_gas`
        // cannot do it, since the charge lives on the parent, not the child.
        let refund_state_gas = match result.refundable_state_gas_charge() {
            // Priced through the same hook the charge went through, so the refund cancels it
            // exactly. A failed lookup surfaces as a fatal external error, like a failed sload.
            Some(charge) => match ctx.state_gas_charge(charge) {
                Some(refund) => Some(refund),
                None => {
                    take_error::<ERROR, _>(ctx.error())?;
                    return Err(ERROR::from_string(
                        "state gas price lookup failed".to_string(),
                    ));
                }
            },
            None => None,
        };
        // The start charge the frame-start hook charged this (parent) frame and declared on the
        // start is refunded on the same condition, whole, by the same two refills a consumer's
        // own take-back uses; it needs no price lookup, since the hook declared the amounts.
        let refund_start_charge = result.refundable_start_charge();

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                let mut out_gas = outcome.gas();
                let ins_result = *outcome.instruction_result();
                let returned_len = outcome.result.output.len();

                let interpreter = &mut self.interpreter;
                let mem_length = outcome.memory_length();
                let mem_start = outcome.memory_start();
                interpreter.return_data.set_buffer(outcome.result.output);

                let target_len = min(mem_length, returned_len);

                if ins_result == InstructionResult::FatalExternalError {
                    panic!("Fatal external error in insert_call_outcome");
                }

                let item = if ins_result.is_ok() {
                    U256::from(1)
                } else {
                    U256::ZERO
                };
                // Safe to push without stack limit check
                let _ = interpreter.stack.push(item);

                // Copy returned data into the parent's memory on success or revert.
                if ins_result.is_ok_or_revert() {
                    interpreter
                        .memory
                        .set(mem_start, &interpreter.return_data.buffer()[..target_len]);
                }

                // Settle the child's gas and merge it into the parent (returns
                // unused regular gas, adds back the reservoir, and propagates state
                // gas / refunds on success).
                handle_reservoir_remaining_gas(
                    ins_result,
                    interpreter.gas.tracker_mut(),
                    out_gas.tracker_mut(),
                );
            }
            FrameResult::Create(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;

                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    interpreter
                        .return_data
                        .set_buffer(outcome.output().to_owned());
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    interpreter.return_data.clear();
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let mut create_gas = *outcome.gas();

                // Settle the child's gas and merge it into the parent (returns
                // unused regular gas, adds back the reservoir, and propagates state
                // gas / refunds on success).
                handle_reservoir_remaining_gas(
                    instruction_result,
                    interpreter.gas.tracker_mut(),
                    create_gas.tracker_mut(),
                );

                let stack_item = if instruction_result.is_ok() {
                    outcome.address.unwrap_or_default().into_word().into()
                } else {
                    U256::ZERO
                };

                // Safe to push without stack limit check
                let _ = interpreter.stack.push(stack_item);
            }
        }

        // Refund the upfront state charge after the child's gas is settled
        // (the settle adds the child's reservoir to what the parent kept).
        if let Some(charge) = refund_state_gas {
            self.interpreter.gas.refill_reservoir(charge);
        }
        if let Some(charge) = refund_start_charge {
            self.interpreter
                .gas
                .tracker_mut()
                .refill_start_charge(charge);
        }

        Ok(())
    }
}

/// Settles a returning child frame's gas and merges it into the parent
/// (EIP-8037 reservoir model).
///
/// First the child *settles its own gas*: a failing frame (revert or halt) rolls
/// its state-gas charges back in last-in-first-out order
/// ([`GasTracker::rollback_state_gas`]) — crediting the spilled portion back to its
/// `remaining` and restoring the reservoir to the value it inherited — and drops
/// its execution refund counter; an exceptional halt additionally consumes the
/// child's regular gas.
///
/// Then the parent *merges* the settled child:
/// - unused regular gas (`remaining`, including any spill returned on revert)
///   flows back to the parent on success or revert; a halt consumes it.
/// - the reservoir, the share of the parent's state-gas pool the child was given
///   at call time, is always added back from the child to what the parent kept
///   (restored to the inherited value on revert/halt). Every schedule defined
///   here forwards the whole reservoir, so the parent kept nothing and its
///   reservoir becomes the child's. Any returned reservoir first unwinds the
///   parent's outstanding spilled state gas in LIFO order.
/// - net state gas, net history gas, the spilled portion, and the refund counter
///   persist only on success; on revert/halt the child's state changes roll back
///   and contribute nothing.
/// - each pool settles as the reservoir does, not as regular gas: the child's
///   rollback gives its draws back to its pools, so what it hands back is its
///   remaining pool on success and the whole share it was given on revert *and on
///   halt*, since a halted frame draws nothing; the parent, which kept what it
///   did not forward, always absorbs it. The child's net draws merge into the
///   parent's on success only, as the history net does, so a later rollback of
///   the parent undoes the child's draws too. With no pool set, every figure here
///   is the unlimited pool going out and coming back.
#[inline]
pub const fn handle_reservoir_remaining_gas(
    instruction_result: InstructionResult,
    parent_gas: &mut GasTracker,
    child_gas: &mut GasTracker,
) {
    // Settle the child's own gas for its stop reason.
    if !instruction_result.is_ok() {
        child_gas.rollback_state_gas();
        child_gas.set_refunded(0);
    }
    if instruction_result.is_halt() {
        // Exceptional halt consumes the child's regular gas (including the spill
        // just credited back by `rollback_state_gas`); the reservoir is left
        // restored to the inherited value for the parent.
        child_gas.spend_all();
    }

    // Merge the settled child into the parent.
    if instruction_result.is_ok_or_revert() {
        parent_gas.erase_cost(child_gas.remaining());
    }
    if instruction_result.is_ok() {
        // Parent may have already charged state gas (e.g. new_account + create)
        // before creating the child frame, so add rather than overwrite. The
        // child's `state_gas_spent` can be negative (EIP-8037 issue #2) when it
        // did more 0→x→0 restorations than 0→x creations; the negative
        // contribution is the parent's matching charge flowing back out.
        parent_gas.set_state_gas_spent(
            parent_gas
                .state_gas_spent()
                .saturating_add(child_gas.state_gas_spent()),
        );
        parent_gas.add_state_gas_spilled(child_gas.state_gas_spilled());
        // History gas rides the same success-only merge, signed for the same reason: a
        // failing child rolled it back along with its state gas, so it contributes
        // nothing. Zero on any chain that never charges history gas.
        parent_gas.set_history_gas_spent(
            parent_gas
                .history_gas_spent()
                .saturating_add(child_gas.history_gas_spent()),
        );
        parent_gas.record_refund(child_gas.refunded());
    }
    parent_gas.absorb_returned_reservoir(child_gas.reservoir());
    parent_gas.absorb_returned_pools(child_gas, instruction_result.is_ok());
}

/// Handles the result of a CREATE operation, including validation and state updates.
///
/// The EIP-8037 upfront CREATE state gas is charged on the parent's tracker by
/// the CREATE/CREATE2 opcode. On child failure (revert/halt/early-fail) it is
/// refunded to the parent in `return_result`. The child frame is NOT allowed to
/// borrow the upfront charge to pay for code deposit: it must cover code deposit
/// state gas from its own reservoir and remaining gas.
///
/// Once the code passed every check and every deposit charge is recorded, and before the
/// checkpoint is committed, the context is asked to admit the deposit
/// ([`ContextTr::admit_code_deposit`]). A refused deposit reverts the checkpoint, puts the
/// frame's gas back to what it was before the deposit charges and ends the creation as a
/// `Revert` with the output the context named.
pub fn return_create<CTX: ContextTr>(
    context: &mut CTX,
    checkpoint: JournalCheckpoint,
    interpreter_result: &mut InterpreterResult,
    address: Address,
) {
    let (_, _, cfg, journal, _, _) = context.all_mut();

    let max_code_size = cfg.max_code_size();
    let is_eip3541_disabled = cfg.is_eip3541_disabled();
    let spec_id = cfg.spec().into();
    let is_amsterdam_eip8037 = cfg.is_amsterdam_eip8037_enabled();
    let gas_params = cfg.gas_params();

    // If return is not ok revert and return.
    if !interpreter_result.result.is_ok() {
        journal.checkpoint_revert(checkpoint);
        return;
    }

    // EIP-170: Contract code size limit to 0x6000 (~25kb)
    // EIP-7954 increased this limit to 0x10000 (64kb).
    // This must be checked BEFORE charging state gas for code deposit,
    // so that oversized code does not incur storage gas costs.
    if spec_id.is_enabled_in(SPURIOUS_DRAGON) && interpreter_result.output.len() > max_code_size {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractSizeLimit;
        return;
    }

    // Host error if present on execution
    // If ok, check contract creation limit and calculate gas deduction on output len.
    //
    // EIP-3541: Reject new contract code starting with the 0xEF byte
    if !is_eip3541_disabled
        && spec_id.is_enabled_in(LONDON)
        && interpreter_result.output.first() == Some(&0xEF)
    {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
        return;
    }

    // The frame's gas before the deposit charges, which a refused deposit gets back.
    let gas_before_deposit = interpreter_result.gas;

    // regular gas for code deposit. It is zero in EIP-8037.
    let gas_for_code = gas_params.code_deposit_cost(interpreter_result.output.len());
    if !interpreter_result.gas.record_regular_cost(gas_for_code) {
        // Record code deposit gas cost and check if we are out of gas.
        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
        // final gas fee for adding the contract code to the state, the contract
        // creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
        if spec_id.is_enabled_in(HOMESTEAD) {
            journal.checkpoint_revert(checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        } else {
            interpreter_result.output = Bytes::new();
        }
    }

    // EIP-8037: Hash cost for deployed bytecode (keccak256)
    // HASH_COST(L) = 6 × ceil(L / 32)
    // Both CREATE and CREATE2 must pay this cost: it covers hashing the deployed code
    // to compute the code_hash stored in the account. CREATE2's existing keccak256 charge
    // (in create2_cost) is for hashing the init code during address derivation, which is
    // a different hash.
    if is_amsterdam_eip8037 {
        let hash_cost = gas_params.keccak256_cost(interpreter_result.output.len());
        if !interpreter_result.gas.record_regular_cost(hash_cost) {
            journal.checkpoint_revert(checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        }
        // State gas for code deposit (EIP-8037).
        // Charged after size check: only code that passes validation incurs state gas cost.
        //
        // Note: This should be last operation before checkpoint commit as spending state before this messes
        // with refilling of state gas.
        //
        // The schedule decides whether deposited code costs state gas at all; the per-byte price
        // comes from the pricing hook, which needs the whole context and so ends the journal
        // borrow above.
        let code_len = interpreter_result.output.len();
        let charges_state_gas = gas_params.code_deposit_state_gas(code_len) > 0;
        // Read before the state-gas charge borrows the context: the value is a flat per-byte
        // price, so it needs nothing but the schedule.
        let history_gas_for_code = gas_params.code_deposit_history_gas(code_len);
        if charges_state_gas {
            let charge = StateGasCharge::units(
                GasId::code_deposit_state_gas(),
                StateGasSite::account(address),
                code_len as u64,
            );
            let state_gas_for_code = context.state_gas_charge(charge);
            let recorded = state_gas_for_code
                .is_some_and(|cost| interpreter_result.gas.record_state_cost(cost));
            if !recorded {
                context.journal_mut().checkpoint_revert(checkpoint);
                interpreter_result.result = if state_gas_for_code.is_some() {
                    InstructionResult::OutOfGas
                } else {
                    // The hook recorded the cause; `EthFrame::return_result` surfaces it.
                    InstructionResult::FatalExternalError
                };
                return;
            }
        }

        // History gas for the same bytes, on the same condition: code that passed validation
        // and is about to be written is also code every node has to carry in a block. The
        // schedule decides whether that costs anything; it is zero everywhere here, so this
        // charge does not exist unless a chain prices history bytes.
        if history_gas_for_code > 0
            && !interpreter_result
                .gas
                .record_history_cost(history_gas_for_code)
        {
            context.journal_mut().checkpoint_revert(checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        }
    }

    if !code_deposit::admit_code_deposit(
        context,
        checkpoint,
        interpreter_result,
        &gas_before_deposit,
        address,
    ) {
        return;
    }

    // If we have enough gas we can commit changes.
    let journal = context.journal_mut();
    journal.checkpoint_commit();

    // Do analysis of bytecode straight away.
    let bytecode = Bytecode::new_legacy(interpreter_result.output.clone());

    // Set code
    journal.set_code(address, bytecode);

    interpreter_result.result = InstructionResult::Return;
}

#[cfg(test)]
mod tests {
    use super::*;
    use interpreter::PoolId;

    #[test]
    fn sibling_refill_restores_parent_regular_gas() {
        const STATE_GAS: u64 = 200;
        const CHILD_GAS: u64 = 500;

        let mut parent = GasTracker::new(1_000, 1_000, 0);

        // P calls A. A creates a slot after the reservoir is exhausted, so the
        // charge spills into regular gas and is absorbed by P on success.
        assert!(parent.record_regular_cost(CHILD_GAS));
        let mut child_a = GasTracker::new(CHILD_GAS, CHILD_GAS, 0);
        assert!(child_a.record_state_cost(STATE_GAS));
        handle_reservoir_remaining_gas(InstructionResult::Stop, &mut parent, &mut child_a);
        assert_eq!(parent.remaining(), 800);
        assert_eq!(parent.reservoir(), 0);
        assert_eq!(parent.state_gas_spent(), STATE_GAS as i64);
        assert_eq!(parent.state_gas_spilled(), STATE_GAS);

        // P then calls B. B clears A's slot, but has no local spill counter, so
        // its refill initially lands in its reservoir.
        assert!(parent.record_regular_cost(CHILD_GAS));
        let mut child_b = GasTracker::new(CHILD_GAS, CHILD_GAS, parent.reservoir());
        child_b.refill_reservoir(STATE_GAS);
        assert_eq!(child_b.reservoir(), STATE_GAS);
        assert_eq!(child_b.state_gas_spilled(), 0);

        // On success, P absorbs B's refill in global LIFO order: the reservoir
        // is moved back to regular gas to unwind A's spilled charge.
        handle_reservoir_remaining_gas(InstructionResult::Stop, &mut parent, &mut child_b);
        assert_eq!(parent.remaining(), 1_000);
        assert_eq!(parent.reservoir(), 0);
        assert_eq!(parent.state_gas_spent(), 0);
        assert_eq!(parent.state_gas_spilled(), 0);
    }

    /// A child with `share` of each of its caller's pools.
    fn child_with_share(parent: &mut GasTracker, share: u64) -> GasTracker {
        let mut child = GasTracker::new(500, 500, 0);
        for pool in PoolId::ALL {
            parent.forward_pool(pool, share);
            child.set_pool_remaining(pool, share);
        }
        child
    }

    /// A tracker with every pool at 1,000.
    fn pools_1000() -> GasTracker {
        let mut tracker = GasTracker::new(1_000, 1_000, 0);
        for pool in PoolId::ALL {
            tracker.set_pool_remaining(pool, 1_000);
        }
        tracker
    }

    /// `(remaining, spent)` of every pool.
    fn pools(tracker: &GasTracker) -> [(u64, i64); PoolId::COUNT] {
        PoolId::ALL.map(|pool| (tracker.pool_remaining(pool), tracker.pool_spent(pool)))
    }

    /// A child that succeeds hands back what it left of each share and its net draws; one that
    /// reverts or halts hands back the whole shares, its draws undone, and no net. The caller
    /// kept what it did not forward either way, and each pool settles alone.
    #[test]
    fn test_the_pools_settle_like_the_reservoir_not_like_regular_gas() {
        for result in [
            InstructionResult::Stop,
            InstructionResult::Revert,
            InstructionResult::OutOfGas,
        ] {
            let mut parent = pools_1000();
            for pool in PoolId::ALL {
                assert!(parent.record_pool(pool, 100));
            }
            let mut child = child_with_share(&mut parent, 800);
            assert_eq!(pools(&parent), [(100, 100); 3], "{result:?}: kept back");
            let [first, second, third] = PoolId::ALL;
            assert!(child.record_pool(first, 300));
            assert!(child.record_pool(second, 1));
            child.refill_pool(third, 10);

            handle_reservoir_remaining_gas(result, &mut parent, &mut child);

            if result.is_ok() {
                assert_eq!(pools(&parent), [(600, 400), (899, 101), (910, 90)]);
                assert_eq!(pools(&child), [(500, 300), (799, 1), (810, -10)]);
            } else {
                assert_eq!(pools(&parent), [(900, 100); 3], "{result:?}");
                assert_eq!(
                    pools(&child),
                    [(800, 0); 3],
                    "{result:?}: a failed child's pools read as its shares"
                );
            }
        }
    }

    /// A caller's rollback undoes the draws of a child that succeeded inside it: B's revert
    /// gives A back the whole share it forwarded, C's draws included.
    #[test]
    fn test_a_callers_rollback_undoes_a_successful_childs_draws() {
        let mut a = pools_1000();
        let mut b = child_with_share(&mut a, 1_000);
        for pool in PoolId::ALL {
            assert!(b.record_pool(pool, 100));
        }
        let mut c = child_with_share(&mut b, 900);
        for pool in PoolId::ALL {
            assert!(c.record_pool(pool, 100));
        }

        handle_reservoir_remaining_gas(InstructionResult::Stop, &mut b, &mut c);
        assert_eq!(pools(&b), [(800, 200); 3]);

        handle_reservoir_remaining_gas(InstructionResult::Revert, &mut a, &mut b);
        assert_eq!(pools(&a), [(1_000, 0); 3]);
    }

    /// A child that gives back a draw its caller made goes below zero, and merging it into the
    /// caller on success nets the two out, as a history refill does; on failure the refill is
    /// undone with the rest.
    #[test]
    fn test_a_childs_refill_of_a_callers_draw_nets_out_on_success_only() {
        for result in [InstructionResult::Stop, InstructionResult::Revert] {
            let mut parent = pools_1000();
            for pool in PoolId::ALL {
                assert!(parent.record_pool(pool, 100));
            }
            let mut child = child_with_share(&mut parent, 900);
            for pool in PoolId::ALL {
                child.refill_pool(pool, 100);
            }
            assert_eq!(pools(&child), [(1_000, -100); 3]);

            handle_reservoir_remaining_gas(result, &mut parent, &mut child);

            let settled = if result.is_ok() {
                (1_000, 0)
            } else {
                (900, 100)
            };
            assert_eq!(pools(&parent), [settled; 3], "{result:?}");
        }
    }

    /// With no pool set, the unlimited pools go out whole and come back, less only what was
    /// drawn and kept, and the regular-gas settlement is what it was.
    #[test]
    fn test_unset_pools_settle_to_the_same_regular_gas() {
        for result in [InstructionResult::Stop, InstructionResult::OutOfGas] {
            let mut parent = GasTracker::new(1_000, 1_000, 0);
            assert!(parent.record_regular_cost(500));
            let mut with_pools = parent;
            let mut child = child_with_share(&mut with_pools, u64::MAX);
            assert!(child.record_regular_cost(200));
            let mut plain_child = child;

            handle_reservoir_remaining_gas(result, &mut with_pools, &mut child);
            handle_reservoir_remaining_gas(result, &mut parent, &mut plain_child);

            assert_eq!(with_pools.remaining(), parent.remaining(), "{result:?}");
            assert_eq!(pools(&with_pools), [(u64::MAX, 0); 3], "{result:?}");
        }
    }

    /// A caller that forwarded a share of its reservoir and kept the rest gets back, on top of
    /// what it kept: what the child left when it succeeded, the whole share when it reverted or
    /// halted, the child's state charges undone. Regular gas settles as it did.
    #[test]
    fn test_the_reservoir_a_child_returns_joins_what_its_caller_kept() {
        for result in [
            InstructionResult::Stop,
            InstructionResult::Revert,
            InstructionResult::OutOfGas,
        ] {
            let mut parent = GasTracker::new(10_000, 10_000, 1_000);
            assert!(parent.record_regular_cost(5_000));
            // 98% of 1,000 forwarded, 20 kept.
            parent.forward_reservoir(980);
            let mut child = GasTracker::new(5_000, 5_000, 980);
            assert!(child.record_state_cost(300));

            handle_reservoir_remaining_gas(result, &mut parent, &mut child);

            let (reservoir, state) = if result.is_ok() {
                (700, 300)
            } else {
                (1_000, 0)
            };
            assert_eq!(parent.reservoir(), reservoir, "{result:?}");
            assert_eq!(parent.state_gas_spent(), state, "{result:?}");
            let regular = if result.is_ok_or_revert() {
                10_000
            } else {
                5_000
            };
            assert_eq!(parent.remaining(), regular, "{result:?}");
        }
    }

    /// A child that spends more than its share spills onto its regular gas, as a frame with no
    /// reservoir left does, though its caller kept part of the reservoir: the share is the
    /// child's whole reservoir. A caller's later rollback restores the reservoir it started with
    /// and gives the spill back to regular gas.
    #[test]
    fn test_a_child_spills_past_its_share_and_a_rollback_restores_the_callers_reservoir() {
        let mut a = GasTracker::new(10_000, 10_000, 1_000);
        assert!(a.record_regular_cost(10_000));
        a.forward_reservoir(1_000);
        let mut b = GasTracker::new(10_000, 10_000, 1_000);
        assert!(b.record_regular_cost(5_000));
        b.forward_reservoir(980);
        let mut c = GasTracker::new(5_000, 5_000, 980);
        assert!(c.record_state_cost(1_000));
        assert_eq!(
            (c.reservoir(), c.remaining(), c.state_gas_spilled()),
            (0, 4_980, 20)
        );

        handle_reservoir_remaining_gas(InstructionResult::Stop, &mut b, &mut c);
        assert_eq!(b.reservoir(), 20, "the part B kept is still there");
        assert_eq!(b.state_gas_spilled(), 20);
        assert_eq!(b.remaining(), 9_980);

        handle_reservoir_remaining_gas(InstructionResult::Revert, &mut a, &mut b);
        assert_eq!(a.reservoir(), 1_000);
        assert_eq!(a.remaining(), 10_000);
        assert_eq!(a.state_gas_spent(), 0);
    }

    /// A caller that kept part of its reservoir holds it beside the spill of a child that spent
    /// past its share, which a whole forward cannot produce. A refill in the caller then keeps
    /// EIP-8037's order, regular gas up to the outstanding spill first and the reservoir after,
    /// whichever paid the charge it undoes: a charge the caller paid from its kept reservoir comes
    /// back as regular gas. The sum of the two is conserved, and a rollback restores the reservoir
    /// the caller started with. With the whole reservoir forwarded, the caller's charge spills
    /// too, and its refill returns where the charge was taken.
    #[test]
    fn test_a_refill_beside_a_kept_reservoir_and_a_childs_spill_credits_regular_gas_first() {
        // (reservoir, remaining, spilled, state gas spent) of a tracker.
        let figures = |gas: &GasTracker| {
            (
                gas.reservoir(),
                gas.remaining(),
                gas.state_gas_spilled(),
                gas.state_gas_spent(),
            )
        };
        for (forwarded, after_child, after_charge, after_refill) in [
            // 98%: B keeps 20 and C spills 20; B's charge comes from its reservoir, the refill
            // goes to its regular gas.
            (
                980,
                (20, 9_980, 20, 1_000),
                (10, 9_980, 20, 1_010),
                (10, 9_990, 10, 1_000),
            ),
            // The whole reservoir: B keeps none and C spills 0; B's charge spills, and the refill
            // undoes that spill.
            (
                1_000,
                (0, 10_000, 0, 1_000),
                (0, 9_990, 10, 1_010),
                (0, 10_000, 0, 1_000),
            ),
        ] {
            // B holds 1,000 of reservoir and forwards 5,000 of regular gas; C charges 1,000.
            let mut b = GasTracker::new(10_000, 10_000, 1_000);
            assert!(b.record_regular_cost(5_000));
            b.forward_reservoir(forwarded);
            let mut c = GasTracker::new(5_000, 5_000, forwarded);
            assert!(c.record_state_cost(1_000));
            handle_reservoir_remaining_gas(InstructionResult::Stop, &mut b, &mut c);
            assert_eq!(figures(&b), after_child, "{forwarded}: after C");

            // B writes a slot, 10 of state gas, and puts it back, a refill of 10.
            assert!(b.record_state_cost(10));
            assert_eq!(figures(&b), after_charge, "{forwarded}: after B's charge");
            b.refill_reservoir(10);
            assert_eq!(figures(&b), after_refill, "{forwarded}: after B's refill");
            assert_eq!(
                b.reservoir() + b.remaining(),
                10_000,
                "{forwarded}: the sum is conserved"
            );

            b.rollback_state_gas();
            assert_eq!(
                (b.reservoir(), b.remaining()),
                (1_000, 10_000),
                "{forwarded}: B's rollback restores what it started with"
            );
        }
    }
}
