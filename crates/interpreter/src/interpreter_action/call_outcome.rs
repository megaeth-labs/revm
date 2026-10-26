use crate::{Gas, InstructionResult, InterpreterResult, OutcomeMark, PoolAmounts, StartCharge};
use core::ops::Range;
use primitives::{Address, Bytes, Log};
use std::vec::Vec;

/// Represents the outcome of a call operation in a virtual machine.
///
/// This struct encapsulates the result of executing an instruction by an interpreter, including
/// the result itself, gas usage information, and the memory offset where output data is stored.
///
/// # Fields
///
/// * `result` - The result of the interpreter's execution, including output data and gas usage.
/// * `memory_offset` - The range in memory where the output data is located.
#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallOutcome {
    /// The result of the interpreter's execution, including output data and gas usage
    pub result: InterpreterResult,
    /// The range in memory where the output data is located
    pub memory_offset: Range<usize>,
    /// Flag to indicate if the call is precompile call.
    /// Used by inspector so it can copy the logs for Inspector::logs call.
    pub was_precompile_called: bool,
    /// Precompile call logs. Needs as revert/halt would delete them from Journal.
    /// So they can't be accessed by inspector.
    pub precompile_call_logs: Vec<Log>,
    /// EIP-8037: copied from `CallInputs::charged_new_account_state_gas`. Tells
    /// the parent frame whether `new_account_state_gas` was upfront-charged on
    /// the parent's tracker for this call, so the parent can refund it when
    /// the call reverts/halts.
    pub charged_new_account_state_gas: bool,
    /// EIP-8037: the account `charged_new_account_state_gas` was priced for.
    ///
    /// The parent re-prices this address when it refunds the charge, so the refund matches what
    /// was taken even when the price varies per account. Meaningless while
    /// `charged_new_account_state_gas` is `false`.
    ///
    /// In a format that carries field names, such as JSON, encodings made before this field existed
    /// decode it as [`Address::ZERO`].
    #[cfg_attr(feature = "serde", serde(default))]
    pub charged_state_gas_address: Address,
    /// Copied from `CallInputs::charged_start`: what the frame-start hook charged the caller for
    /// this call and the call's failure gives back, so the parent can refund it when the call
    /// does not succeed.
    ///
    /// In a format that carries field names, such as JSON, encodings made before this field existed
    /// decode it as no charge.
    #[cfg_attr(feature = "serde", serde(default))]
    pub charged_start: StartCharge,
    /// The start revm marked this outcome for before an inspector's end callbacks
    /// ([`OutcomeMark`]): none on an outcome a constructor builds. No part of the outcome's value:
    /// equality, the debug output and the encoding leave it out, and a decoded outcome carries
    /// none.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub start_mark: OutcomeMark,
}

impl core::fmt::Debug for CallOutcome {
    /// The output the derive gives the outcome's other fields, in their order; the mark is left
    /// out.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CallOutcome")
            .field("result", &self.result)
            .field("memory_offset", &self.memory_offset)
            .field("was_precompile_called", &self.was_precompile_called)
            .field("precompile_call_logs", &self.precompile_call_logs)
            .field(
                "charged_new_account_state_gas",
                &self.charged_new_account_state_gas,
            )
            .field("charged_state_gas_address", &self.charged_state_gas_address)
            .field("charged_start", &self.charged_start)
            .finish()
    }
}

impl CallOutcome {
    /// Constructs a new [`CallOutcome`].
    ///
    /// Creates an instance of [`CallOutcome`] with the given interpreter result and memory offset.
    ///
    /// # Arguments
    ///
    /// * `result` - The result of the interpreter's execution.
    /// * `memory_offset` - The range in memory indicating where the output data is stored.
    pub const fn new(result: InterpreterResult, memory_offset: Range<usize>) -> Self {
        Self {
            result,
            memory_offset,
            was_precompile_called: false,
            precompile_call_logs: Vec::new(),
            charged_new_account_state_gas: false,
            charged_state_gas_address: Address::ZERO,
            charged_start: StartCharge::new(0, PoolAmounts::ZERO),
            start_mark: OutcomeMark::NONE,
        }
    }

    /// Constructs a new [`CallOutcome`] for an out-of-gas error.
    ///
    /// # Arguments
    ///
    /// * `gas_limit` - The gas limit that was exceeded.
    /// * `memory_offset` - The range in memory indicating where the output data is stored.
    pub fn new_oog(gas_limit: u64, memory_offset: Range<usize>, reservoir: u64) -> Self {
        Self::new(
            InterpreterResult::new_oog(gas_limit, reservoir),
            memory_offset,
        )
    }

    /// Returns a reference to the instruction result.
    ///
    /// Provides access to the result of the executed instruction.
    ///
    /// # Returns
    ///
    /// A reference to the [`InstructionResult`].
    pub const fn instruction_result(&self) -> &InstructionResult {
        &self.result.result
    }

    /// Returns the gas usage information.
    ///
    /// Provides access to the gas usage details of the executed instruction.
    ///
    /// # Returns
    ///
    /// An instance of [`Gas`] representing the gas usage.
    pub const fn gas(&self) -> Gas {
        self.result.gas
    }

    /// Returns a reference to the output data.
    ///
    /// Provides access to the output data generated by the executed instruction.
    ///
    /// # Returns
    ///
    /// A reference to the output data as [`Bytes`].
    pub const fn output(&self) -> &Bytes {
        &self.result.output
    }

    /// Returns the start position of the memory offset.
    ///
    /// Provides the starting index of the memory range where the output data is stored.
    ///
    /// # Returns
    ///
    /// The starting index of the memory offset as [`usize`].
    pub const fn memory_start(&self) -> usize {
        self.memory_offset.start
    }

    /// Returns the length of the memory range.
    ///
    /// Provides the length of the memory range where the output data is stored.
    ///
    /// # Returns
    ///
    /// The length of the memory range as [`usize`].
    pub fn memory_length(&self) -> usize {
        self.memory_offset.len()
    }
}
