//! This module defines some types used for revm metrics.
use serde::{Deserialize, Serialize};

use crate::time_utils;

pub const STEP_LEN: usize = 4;
pub const SLOAD_OPCODE_TIME_STEP: [u64; STEP_LEN] = [1, 10, 100, u64::MAX];

/// The OpcodeRecord contains all performance information for opcode executions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcodeRecord {
    /// The abscissa is opcode type, tuple means: (opcode counter, time, gas).
    #[serde(with = "serde_arrays")]
    pub opcode_record: [(u64, u64, i128); 256],
    /// tuple means:(the ladder of sload opcode excution time, sload counter).
    #[serde(with = "serde_arrays")]
    pub sload_opcode_record: [(u64, u64); STEP_LEN],
    /// The total time (cpu cycles) of all opcode.
    pub total_time: u64,
    /// Update flag.
    pub is_updated: bool,
}

impl Default for OpcodeRecord {
    fn default() -> Self {
        let sload_opcode_record_init = SLOAD_OPCODE_TIME_STEP.map(|v| (v, 0));
        Self {
            opcode_record: [(0, 0, 0); 256],
            sload_opcode_record: sload_opcode_record_init,
            total_time: 0,
            is_updated: false,
        }
    }
}

impl OpcodeRecord {
    /// Update this struct with the other's data.
    pub fn update(&mut self, other: &mut OpcodeRecord) {
        if !other.is_updated {
            return;
        }

        self.total_time = self
            .total_time
            .checked_add(other.total_time)
            .expect("overflow");

        if !self.is_updated {
            self.opcode_record = std::mem::replace(&mut other.opcode_record, self.opcode_record);
            self.sload_opcode_record =
                std::mem::replace(&mut other.sload_opcode_record, self.sload_opcode_record);
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

        for index in 0..self.sload_opcode_record.len() {
            self.sload_opcode_record[index].1 = self.sload_opcode_record[index]
                .1
                .checked_add(other.sload_opcode_record[index].1)
                .expect("overflow");
        }
    }

    /// Record sload duration percentile.
    pub fn add_sload_opcode_record(&mut self, op_time: u64) {
        for index in 0..SLOAD_OPCODE_TIME_STEP.len() {
            if op_time < SLOAD_OPCODE_TIME_STEP[index] {
                self.sload_opcode_record[index].1 = self.sload_opcode_record[index]
                    .1
                    .checked_add(1)
                    .expect("overflow");
                return;
            }
        }
    }

    pub fn not_empty(&self) -> bool {
        self.is_updated
    }
}

const US_PENALTY_STEP_SIZE: usize = 200;
const NS_PENALTY_STEP_SIZE: usize = 10;
/// The additional cost (cpu cycles) incurred when CacheDb is not hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct CacheMissesPenalty {
    pub time: u64,
    #[serde(with = "serde_arrays")]
    pub us_percentile: [u64; US_PENALTY_STEP_SIZE],
    #[serde(with = "serde_arrays")]
    pub ns_percentile: [u64; NS_PENALTY_STEP_SIZE],
}

impl Default for CacheMissesPenalty {
    fn default() -> Self {
        CacheMissesPenalty {
            time: 0,
            us_percentile: [0; US_PENALTY_STEP_SIZE],
            ns_percentile: [0; NS_PENALTY_STEP_SIZE],
        }
    }
}

impl CacheMissesPenalty {
    pub fn update(&mut self, other: &Self) {
        self.time = self.time.checked_add(other.time).expect("overflow");

        for index in 0..US_PENALTY_STEP_SIZE {
            self.us_percentile[index] = self.us_percentile[index]
                .checked_add(other.us_percentile[index])
                .expect("overflow");
        }
        for index in 0..NS_PENALTY_STEP_SIZE {
            self.ns_percentile[index] = self.ns_percentile[index]
                .checked_add(other.ns_percentile[index])
                .expect("overflow");
        }
    }

    pub fn percentile(&mut self, time_in_ns: f64) {
        if time_in_ns >= 1000.0 {
            let mut index = (time_in_ns / 1000.0) as usize;
            if index > US_PENALTY_STEP_SIZE - 1 {
                index = US_PENALTY_STEP_SIZE - 1;
            }
            self.us_percentile[index] = self.us_percentile[index].checked_add(1).expect("overflow");
        } else {
            self.us_percentile[0] = self.us_percentile[0].checked_add(1).expect("overflow");
            let index = (time_in_ns / 100.0) as usize;
            self.ns_percentile[index] = self.ns_percentile[index].checked_add(1).expect("overflow");
        }
    }
}

/// CacheDbRecord records the relevant information of CacheDb hits during the execution process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheDbRecord {
    /// The number of cache hits when accessing CacheDB.
    pub hits: u64,
    /// The number of cache miss when accessing CacheDB.
    pub misses: u64,
    /// The additional cost incurred when accessing CacheDb without a cache hit.
    pub penalty: CacheMissesPenalty,
}

impl CacheDbRecord {
    /// Update this struct with the other's data.
    pub fn update(&mut self, other: &Self) {
        self.hits = self.hits.checked_add(other.hits).expect("overflow");
        self.misses = self.misses.checked_add(other.misses).expect("overflow");
        self.penalty.update(&other.penalty);
    }

    /// When hit, increase the number of hits count.
    pub(super) fn hit(&mut self) {
        self.hits = self.hits.checked_add(1).expect("overflow");
    }

    /// When a miss occurs, it is necessary to increase the number of misses count,
    /// record the increased penalty, and record the distribution of penalty.
    pub(super) fn miss(&mut self, penalty: u64) {
        self.misses = self.misses.checked_add(1).expect("overflow");
        self.penalty.time = self.penalty.time.checked_add(penalty).expect("overflow");
        self.penalty
            .percentile(time_utils::convert_cycles_to_ns_f64(penalty));
    }
}
