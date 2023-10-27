pub mod analysis;
mod contract;
pub(crate) mod memory;
mod stack;

pub use analysis::BytecodeLocked;
pub use contract::Contract;
pub use memory::Memory;
pub use stack::Stack;

use crate::primitives::{Bytes, Spec};
use crate::{
    instructions::{eval, InstructionResult},
    Gas, Host,
};
use core::ops::Range;

pub const STACK_LIMIT: u64 = 1024;
pub const CALL_STACK_LIMIT: u64 = 1024;

/// EIP-170: Contract code size limit
/// By default limit is 0x6000 (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;
/// EIP-3860: Limit and meter initcode
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

pub struct Interpreter {
    /// Instruction pointer.
    pub instruction_pointer: *const u8,
    /// Return is main control flag, it tell us if we should continue interpreter or break from it
    pub instruction_result: InstructionResult,
    /// left gas. Memory gas can be found in Memory field.
    pub gas: Gas,
    /// Memory.
    pub memory: Memory,
    /// Stack.
    pub stack: Stack,
    /// After call returns, its return data is saved here.
    pub return_data_buffer: Bytes,
    /// Return value.
    pub return_range: Range<usize>,
    /// Is interpreter call static.
    pub is_static: bool,
    /// Contract information and invoking data
    pub contract: Contract,
    /// Memory limit. See [`crate::CfgEnv`].
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
    /// Used for record duration of instruction. This opcode_code means: (counter, time),
    #[cfg(feature = "enable_opcode_metrics")]
    // pub opcode_record: [(u64, std::time::Duration); 256],
    pub opcode_record: revm_utils::types::OpcodeRecord,
    /// Interpreter start time.
    #[cfg(feature = "enable_opcode_metrics")]
    start_time: Option<minstant::Instant>,
    /// Used for record time.
    #[cfg(feature = "enable_opcode_metrics")]
    pre_time: Option<minstant::Instant>,
    /// Enable metric record.
    #[cfg(feature = "enable_opcode_metrics")]
    enable_metric_record: bool,
}

impl Interpreter {
    /// Current opcode
    pub fn current_opcode(&self) -> u8 {
        unsafe { *self.instruction_pointer }
    }

    /// Create new interpreter
    #[cfg(not(feature = "enable_opcode_metrics"))]
    pub fn new(contract: Contract, gas_limit: u64, is_static: bool) -> Self {
        #[cfg(not(feature = "memory_limit"))]
        {
            Self {
                instruction_pointer: contract.bytecode.as_ptr(),
                return_range: Range::default(),
                memory: Memory::new(),
                stack: Stack::new(),
                return_data_buffer: Bytes::new(),
                contract,
                instruction_result: InstructionResult::Continue,
                is_static,
                gas: Gas::new(gas_limit),
            }
        }

        #[cfg(feature = "memory_limit")]
        {
            Self::new_with_memory_limit(contract, gas_limit, is_static, u64::MAX)
        }
    }

    /// Create new interpreter
    #[cfg(feature = "enable_opcode_metrics")]
    pub fn new(
        contract: Contract,
        gas_limit: u64,
        is_static: bool,
        enable_metric_record: bool,
    ) -> Self {
        #[cfg(not(feature = "memory_limit"))]
        {
            Self {
                instruction_pointer: contract.bytecode.as_ptr(),
                return_range: Range::default(),
                memory: Memory::new(),
                stack: Stack::new(),
                return_data_buffer: Bytes::new(),
                contract,
                instruction_result: InstructionResult::Continue,
                is_static,
                gas: Gas::new(gas_limit),
                opcode_record: revm_utils::types::OpcodeRecord::default(),
                start_time: None,
                pre_time: None,
                enable_metric_record,
            }
        }

        #[cfg(feature = "memory_limit")]
        {
            Self::new_with_memory_limit(contract, gas_limit, is_static, u64::MAX)
        }
    }

    #[cfg(not(feature = "enable_opcode_metrics"))]
    #[cfg(feature = "memory_limit")]
    pub fn new_with_memory_limit(
        contract: Contract,
        gas_limit: u64,
        is_static: bool,
        memory_limit: u64,
    ) -> Self {
        Self {
            instruction_pointer: contract.bytecode.as_ptr(),
            return_range: Range::default(),
            memory: Memory::new(),
            stack: Stack::new(),
            return_data_buffer: Bytes::new(),
            contract,
            instruction_result: InstructionResult::Continue,
            is_static,
            gas: Gas::new(gas_limit),
            memory_limit,
        }
    }

    #[cfg(all(feature = "memory_limit", feature = "enable_opcode_metrics"))]
    pub fn new_with_memory_limit(
        contract: Contract,
        gas_limit: u64,
        is_static: bool,
        memory_limit: u64,
        enable_metric_record: bool,
    ) -> Self {
        Self {
            instruction_pointer: contract.bytecode.as_ptr(),
            return_range: Range::default(),
            memory: Memory::new(),
            stack: Stack::new(),
            return_data_buffer: Bytes::new(),
            contract,
            instruction_result: InstructionResult::Continue,
            is_static,
            gas: Gas::new(gas_limit),
            memory_limit,
            opcode_record: revm_utils::types::OpcodeRecord::default(),
            start_time: None,
            pre_time: None,
            enable_metric_record,
        }
    }

    pub fn contract(&self) -> &Contract {
        &self.contract
    }

    pub fn gas(&self) -> &Gas {
        &self.gas
    }

    /// Reference of interpreter memory.
    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    /// Reference of interpreter stack.
    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    /// Return a reference of the program counter.
    pub fn program_counter(&self) -> usize {
        // Safety: this is just subtraction of pointers, it is safe to do.
        unsafe {
            self.instruction_pointer
                .offset_from(self.contract.bytecode.as_ptr()) as usize
        }
    }

    /// Execute next instruction
    #[inline(always)]
    pub fn step<H: Host, SPEC: Spec>(&mut self, host: &mut H) {
        // step.
        let opcode = unsafe { *self.instruction_pointer };
        // Safety: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.instruction_pointer = unsafe { self.instruction_pointer.offset(1) };

        #[cfg(not(feature = "enable_opcode_metrics"))]
        eval::<H, SPEC>(opcode, self, host);

        #[cfg(feature = "enable_opcode_metrics")]
        {
            let mut gas_used = 0u64;
            let mut gas_refund = 0i64;
            eval::<H, SPEC>(opcode, self, host, &mut gas_used, &mut gas_refund);
            self.opcode_record.opcode_record[opcode as usize].2 = self.opcode_record.opcode_record
                [opcode as usize]
                .2
                .checked_add(gas_used.into())
                .expect("overflow");
            if gas_refund != 0 {
                self.opcode_record.opcode_record[opcode as usize].2 =
                    self.opcode_record.opcode_record[opcode as usize]
                        .2
                        .checked_add(gas_refund.into())
                        .expect("overflow");
            }

            if self.enable_metric_record {
                let now = minstant::Instant::now();
                self.opcode_record.opcode_record[opcode as usize].0 =
                    self.opcode_record.opcode_record[opcode as usize]
                        .0
                        .checked_add(1)
                        .expect("overflow");
                let duration = now
                    .checked_duration_since(self.pre_time.expect("pre time is empty"))
                    .expect("overflow");
                self.opcode_record.opcode_record[opcode as usize].1 =
                    self.opcode_record.opcode_record[opcode as usize]
                        .1
                        .checked_add(duration)
                        .expect("overflow");
                self.opcode_record.is_updated = true;
                self.pre_time = Some(now);
            }
        }
    }

    /// loop steps until we are finished with execution
    pub fn run<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> InstructionResult {
        #[cfg(feature = "enable_opcode_metrics")]
        {
            let now = minstant::Instant::now();
            self.start_time = Some(now);
            self.pre_time = Some(now);
        }
        while self.instruction_result == InstructionResult::Continue {
            self.step::<H, SPEC>(host)
        }
        #[cfg(feature = "enable_opcode_metrics")]
        {
            let now = minstant::Instant::now();
            self.opcode_record.total_time = now
                .checked_duration_since(self.start_time.expect("start time is empty"))
                .expect("overflow");
        }

        self.instruction_result
    }

    /// loop steps until we are finished with execution
    pub fn run_inspect<H: Host, SPEC: Spec>(&mut self, host: &mut H) -> InstructionResult {
        while self.instruction_result == InstructionResult::Continue {
            // step
            let ret = host.step(self, self.is_static);
            if ret != InstructionResult::Continue {
                return ret;
            }
            self.step::<H, SPEC>(host);

            // step ends
            let ret = host.step_end(self, self.is_static, self.instruction_result);
            if ret != InstructionResult::Continue {
                return ret;
            }
        }
        self.instruction_result
    }

    /// Copy and get the return value of the interpreter, if any.
    pub fn return_value(&self) -> Bytes {
        // if start is usize max it means that our return len is zero and we need to return empty
        if self.return_range.start == usize::MAX {
            Bytes::new()
        } else {
            Bytes::copy_from_slice(self.memory.get_slice(
                self.return_range.start,
                self.return_range.end - self.return_range.start,
            ))
        }
    }
}
