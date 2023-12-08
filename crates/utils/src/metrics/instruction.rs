//! This module defines a structure to support the recording of metrics
//! during instruction execution.
use super::types::*;
use crate::time_utils::{convert_cycles_to_ns_f64, instant::Instant};

/// This struct is used to record information during instruction execution
/// and finally stores the data in the opcode_record field.
#[derive(Debug, Default)]
pub(crate) struct InstructionMetricRecoder {
    record: OpcodeRecord,
    start_time: Option<Instant>,
    pre_time: Option<Instant>,
    started: bool,
}

impl InstructionMetricRecoder {
    /// Start record.
    pub(crate) fn start_record(&mut self) {
        let now = Instant::now();

        if !self.started {
            self.start_time = Some(now);
            self.pre_time = Some(now);
        }
        self.started = true;
    }

    /// Record opcode execution information, recording: count, time and sload percentile.
    pub(crate) fn record_op(&mut self, opcode: u8) {
        let now = Instant::now();

        // calculate count
        self.record.opcode_record[opcode as usize].0 = self.record.opcode_record[opcode as usize]
            .0
            .checked_add(1)
            .expect("overflow");

        // calculate time
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

        // SLOAD = 0x54,
        // statistical percentile of sload duration
        if opcode == 0x54 {
            self.record
                .add_sload_opcode_record(convert_cycles_to_ns_f64(cycles));
        }

        self.record.is_updated = true;
    }

    /// Retrieve the records of opcode execution, which will be reset after retrieval.
    pub(crate) fn get_record(&mut self) -> OpcodeRecord {
        self.start_time = None;
        self.pre_time = None;
        self.started = false;
        std::mem::replace(&mut self.record, OpcodeRecord::default())
    }

    /// Record the gas consumption during opcode execution.
    pub(crate) fn record_gas(&mut self, opcode: u8, gas_used: u64) {
        // calculate gas
        self.record.opcode_record[opcode as usize].2 = self.record.opcode_record[opcode as usize]
            .2
            .checked_add(gas_used.into())
            .expect("overflow");
    }
}
