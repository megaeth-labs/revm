use std::time::Duration;

use serde::{Deserialize, Serialize};

pub type RevmMetricRecord = OpcodeRecord;

pub const STEP_LEN: usize = 4;
pub const SLOAD_OPCODE_TIME_STEP: [u128; STEP_LEN] = [1, 10, 100, u128::MAX];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcodeRecord {
    /// The abscissa is opcode type, tuple means: (opcode counter, time, gas).
    #[serde(with = "serde_arrays")]
    pub opcode_record: [(u64, Duration, i128); 256],
    /// tuple means:(the ladder of sload opcode excution time, sload opcode counter).
    #[serde(with = "serde_arrays")]
    pub sload_opcode_record: [(u128, u128); STEP_LEN],
    /// The total time of all opcode.
    pub total_time: Duration,
    pub is_updated: bool,
}

impl Default for OpcodeRecord {
    fn default() -> Self {
        let sload_opcode_record_init = SLOAD_OPCODE_TIME_STEP.map(|v| (v, 0));
        Self {
            opcode_record: [(0, Duration::default(), 0); 256],
            sload_opcode_record: sload_opcode_record_init,
            total_time: Duration::default(),
            is_updated: false,
        }
    }
}

impl OpcodeRecord {
    pub fn update(&mut self, other: &mut OpcodeRecord) {
        if !other.is_updated {
            return;
        }

        if !self.is_updated {
            self.opcode_record = std::mem::replace(&mut other.opcode_record, self.opcode_record);
            self.sload_opcode_record = std::mem::replace(&mut other.sload_opcode_record, self.sload_opcode_record);
            self.is_updated = true;
            return;
        }

        for i in 0..256 {
            self.opcode_record[i].0 = self.opcode_record[i]
                .0
                .checked_add(other.opcode_record[i].0)
                .expect("overflow");
            self.opcode_record[i].1 = self.opcode_record[i]
                .1
                .checked_add(other.opcode_record[i].1)
                .expect("overflow");
            self.opcode_record[i].2 = self.opcode_record[i]
                .2
                .checked_add(other.opcode_record[i].2)
                .expect("overflow");
        }

        self.total_time = self
            .total_time
            .checked_add(other.total_time)
            .expect("overflow");

        for index in 0..self.sload_opcode_record.len() {
            self.sload_opcode_record[index].1 = self.sload_opcode_record[index]
                .1
                .checked_add(other.sload_opcode_record[index].1)
                .expect("overflow");
        }
    }

    pub fn add_sload_opcode_record(&mut self, op_time: u128) {
        for index in 0..SLOAD_OPCODE_TIME_STEP.len() {
            if op_time <= SLOAD_OPCODE_TIME_STEP[index] {
                self.sload_opcode_record[index].1 = self.sload_opcode_record[index].1 + 1;
                return;
            }
        }
    }

    pub fn not_empty(&self) -> bool {
        self.is_updated
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheHits {
    pub hits_in_block_hash: u64,
    pub hits_in_basic: u64,
    pub hits_in_storage: u64,
    pub hits_in_code_by_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheMisses {
    pub misses_in_block_hash: u64,
    pub misses_in_basic: u64,
    pub misses_in_storage: u64,
    pub misses_in_code_by_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheMissesPenalty {
    pub penalty_in_block_hash: Duration,
    pub penalty_in_basic: Duration,
    pub penalty_in_storage: Duration,
    pub penalty_in_code_by_hash: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheDbRecord {
    pub hits: CacheHits,
    pub misses: CacheMisses,
    pub penalty: CacheMissesPenalty,
}
