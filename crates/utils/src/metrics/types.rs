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

/// This type represents in which function the access cache is accessed.
#[derive(Copy, Clone)]
pub enum Function {
    Basic = 0,
    CodeByHash,
    Storage,
    BlockHash,
    LoadAccount,
}
/// This structure records the number of times cache hits/misses are accessed in each function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct AccessStats {
    /// This array is used to store the number of hits/misses/penalty in each function,
    /// and the index of the function corresponds to the order of the FunctionType.
    #[serde(with = "serde_arrays")]
    pub function: [u64; 5],
}

impl AccessStats {
    pub fn update(&mut self, other: &Self) {
        for i in 0..self.function.len() {
            self.function[i] = self.function[i]
                .checked_add(other.function[i])
                .expect("overflow");
        }
    }

    fn increment(&mut self, function: Function) {
        self.add(function, 1);
    }

    fn add(&mut self, function: Function, value: u64) {
        let index = function as usize;
        self.function[index] = self.function[index].checked_add(value).expect("overflow");
    }
}

const US_PENALTY_STEP_SIZE: usize = 200;
const NS_PENALTY_STEP_SIZE: usize = 40;
/// The additional cost (cpu cycles) incurred when CacheDb is not hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct MissesPenalty {
    // Record the penalty when each function hits the cache.
    pub time: AccessStats,
    /// Record the time distribution at a subtle level.
    #[serde(with = "serde_arrays")]
    pub us_percentile: [u64; US_PENALTY_STEP_SIZE],
    /// Record the time distribution at a nanosecond level.
    #[serde(with = "serde_arrays")]
    pub ns_percentile: [u64; NS_PENALTY_STEP_SIZE],
}

impl Default for MissesPenalty {
    fn default() -> Self {
        MissesPenalty {
            time: AccessStats::default(),
            us_percentile: [0; US_PENALTY_STEP_SIZE],
            ns_percentile: [0; NS_PENALTY_STEP_SIZE],
        }
    }
}

impl MissesPenalty {
    pub fn update(&mut self, other: &Self) {
        self.time.update(&other.time);

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

    fn percentile(&mut self, time_in_ns: f64) {
        // Record the time distribution at a subtle level.
        let mut index = (time_in_ns / 1000.0) as usize;
        if index > US_PENALTY_STEP_SIZE - 1 {
            index = US_PENALTY_STEP_SIZE - 1;
        }
        self.us_percentile[index] = self.us_percentile[index].checked_add(1).expect("overflow");

        // When the time is less than 4 us, record the distribution of time at the nanosecond level.
        if time_in_ns < 4000.0 {
            let index = (time_in_ns / 100.0) as usize;
            self.ns_percentile[index] = self.ns_percentile[index].checked_add(1).expect("overflow");
        }
    }
}

/// CacheDbRecord records the relevant information of CacheDb hits during the execution process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheDbRecord {
    /// The number of cache hits when accessing CacheDB.
    hits: AccessStats,
    /// The number of cache miss when accessing CacheDB.
    misses: AccessStats,
    /// The additional cost incurred when accessing CacheDb without a cache hit.
    penalty: MissesPenalty,
}

impl CacheDbRecord {
    /// Update this struct with the other's data.
    pub fn update(&mut self, other: &Self) {
        self.hits.update(&other.hits);
        self.misses.update(&other.misses);
        self.penalty.update(&other.penalty);
    }

    /// Returns the total number of times cache has been accessed in each function.
    pub fn access_count(&self) -> AccessStats {
        let mut stats = self.hits;
        stats.update(&self.misses);
        stats
    }

    /// Returns the number of hits in each function.
    pub fn hit_stats(&self) -> AccessStats {
        self.hits
    }

    /// Returns the number of misses in each function.
    pub fn miss_stats(&self) -> AccessStats {
        self.misses
    }

    /// Return the penalties missed in each function and their distribution.
    pub fn penalty_stats(&self) -> MissesPenalty {
        self.penalty
    }

    /// When hit, increase the number of hits count.
    pub(super) fn hit(&mut self, function: Function) {
        self.hits.increment(function);
    }

    /// When a miss occurs, it is necessary to increase the number of misses count,
    /// record the increased penalty, and record the distribution of penalty.
    pub(super) fn miss(&mut self, function: Function, penalty: u64) {
        self.misses.increment(function);
        self.penalty.time.add(function, penalty);
        self.penalty
            .percentile(time_utils::convert_cycles_to_ns_f64(penalty));
    }
}
