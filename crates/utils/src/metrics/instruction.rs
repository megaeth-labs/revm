//! This module defines a structure to support the recording of metrics
//! during instruction execution.
use super::types::*;
use crate::time_utils::{convert_cycles_to_ns_f64, instant::Instant};

/// This struct is used to record information during instruction execution
/// and finally stores the data in the opcode_record field.
#[derive(Debug, Default)]
pub(super) struct InstructionMetricRecoder {
    record: OpcodeRecord,
    start_time: Option<Instant>,
    pre_time: Option<Instant>,
    pre_opcode: Option<u8>,
    started: bool,
}

impl InstructionMetricRecoder {
    /// Start record.
    pub(super) fn start_record(&mut self) {
        let now = Instant::now();

        if !self.started {
            self.start_time = Some(now);
            self.pre_time = Some(now);
        } else if self.has_call_opcode() {
            let opcode = self.pre_opcode.expect("pre code is empty");
            self.record_time(now, opcode);
            self.record.add_additional_count(opcode, 1);
        }
        self.started = true;
    }

    /// Determine whether an instruction is a call related instruction.
    fn has_call_opcode(&self) -> bool {
        if let Some(opcode) = self.pre_opcode {
            match opcode {
                // CALL | CALLCODE | DELEGATECALL | STATICCALL
                0xF1 | 0xF2 | 0xF4 | 0xFA => return true,
                // other opcode
                _ => return false,
            }
        }
        false
    }

    /// Called before each instruction execution, it is mainly used to handle
    /// the situation that the INTERPRETER will be created circularly when the
    /// call related instructions are executed.
    pub(super) fn record_before_op(&mut self, opcode: u8) {
        self.pre_opcode = Some(opcode)
    }

    /// Record the time taken for instruction execution.
    fn record_time(&mut self, now: Instant, opcode: u8) -> u64 {
        let cycles = now
            .checked_cycles_since(self.pre_time.expect("pre time is empty"))
            .expect("overflow");
        self.record.opcode_record[opcode as usize].1 = self.record.opcode_record[opcode as usize]
            .1
            .checked_add(cycles.into())
            .expect("overflow");
        self.pre_time = Some(now);

        // update total time
        self.record.total_time = now
            .checked_cycles_since(self.start_time.expect("start time is empty"))
            .expect("overflow")
            .into();

        cycles
    }

    /// Record opcode execution information, recording: count, time and sload percentile.
    pub(super) fn record_op(&mut self, opcode: u8) {
        let now = Instant::now();

        // record count
        self.record.opcode_record[opcode as usize].0 = self.record.opcode_record[opcode as usize]
            .0
            .checked_add(1)
            .expect("overflow");

        // record time
        let cycles = self.record_time(now, opcode);

        // SLOAD = 0x54,
        // statistical percentile of sload duration
        if opcode == 0x54 {
            self.record
                .add_sload_opcode_record(convert_cycles_to_ns_f64(cycles));
        }

        self.record.is_updated = true;
    }

    /// Retrieve the records of opcode execution, which will be reset after retrieval.
    pub(super) fn get_record(&mut self) -> OpcodeRecord {
        self.start_time = None;
        self.pre_time = None;
        self.pre_opcode = None;
        self.started = false;
        std::mem::replace(&mut self.record, OpcodeRecord::default())
    }

    /// Record the gas consumption during opcode execution.
    pub(super) fn record_gas(&mut self, opcode: u8, gas_used: u64) {
        // calculate gas
        self.record.opcode_record[opcode as usize].2 = self.record.opcode_record[opcode as usize]
            .2
            .checked_add(gas_used.into())
            .expect("overflow");
    }
}
