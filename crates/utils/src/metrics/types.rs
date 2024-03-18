//! This module defines some types used for revm metrics.
use serde::{Deserialize, Serialize};

use crate::time_utils;

const STEP_IN_US: usize = 1;
const STEP_IN_NS: usize = 100;

const US_SPAN_SIZE: usize = 200;
const NS_SPAN_SIZE: usize = 40;
const MAX_ARRAY_SIZE: usize = 200;
/// This is a structure for statistical time distribution, which records the
/// distribution of time from two levels: subtle and nanosecond.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]

pub struct TimeDistributionStats {
    /// The subtle range of statistical distribution, step is STEP_IN_US.
    pub span_in_us: usize,
    /// The nanosecond range of statistical distribution, step is STEP_IN_NS.
    pub span_in_ns: usize,
    /// Record the time distribution at a subtle level.
    #[serde(with = "serde_arrays")]
    pub us_percentile: [u64; MAX_ARRAY_SIZE],
    /// Record the time distribution at a nanosecond level.
    #[serde(with = "serde_arrays")]
    pub ns_percentile: [u64; MAX_ARRAY_SIZE],
}

impl Default for TimeDistributionStats {
    fn default() -> Self {
        Self::new(US_SPAN_SIZE, NS_SPAN_SIZE)
    }
}

impl TimeDistributionStats {
    pub fn new(span_in_us: usize, span_in_ns: usize) -> Self {
        TimeDistributionStats {
            span_in_us,
            span_in_ns,
            us_percentile: [0; MAX_ARRAY_SIZE],
            ns_percentile: [0; MAX_ARRAY_SIZE],
        }
    }

    pub fn update(&mut self, other: &TimeDistributionStats) {
        for index in 0..self.span_in_us {
            self.us_percentile[index] = self.us_percentile[index]
                .checked_add(other.us_percentile[index])
                .expect("overflow");
        }
        for index in 0..self.span_in_ns {
            self.ns_percentile[index] = self.ns_percentile[index]
                .checked_add(other.ns_percentile[index])
                .expect("overflow");
        }
    }

    pub fn record(&mut self, time_in_ns: f64) {
        // Record the time distribution at a subtle level.
        let mut index = (time_in_ns / (1000.0 * STEP_IN_US as f64)) as usize;
        if index > self.span_in_us - 1 {
            index = self.span_in_us - 1;
        }
        self.us_percentile[index] = self.us_percentile[index].checked_add(1).expect("overflow");

        // When the time is less than 4 us, record the distribution of time at the nanosecond level.
        if time_in_ns < (self.span_in_ns * STEP_IN_NS) as f64 {
            let index = (time_in_ns / STEP_IN_NS as f64) as usize;
            self.ns_percentile[index] = self.ns_percentile[index].checked_add(1).expect("overflow");
        }
    }
}

const CALL_OPCODE_LEN: usize = 4;
/// The OpcodeRecord contains all performance information for opcode executions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcodeRecord {
    /// The abscissa is opcode type, tuple means: (opcode counter, time, gas).
    #[serde(with = "serde_arrays")]
    pub opcode_record: [(u64, u64, i128); 256],
    /// Record the time distribution of the sload.
    pub sload_percentile: TimeDistributionStats,
    /// The total time (cpu cycles) of all opcode.
    pub total_time: u64,
    /// Update flag.
    pub is_updated: bool,
    /// Additional rdtsc counts that may be added when measuring call related instructions.
    /// array means: (call, call_code, delegate_call, static_call)
    pub additional_count: [u64; CALL_OPCODE_LEN],
}

impl Default for OpcodeRecord {
    fn default() -> Self {
        let sload_percentile = TimeDistributionStats::new(US_SPAN_SIZE, NS_SPAN_SIZE);
        Self {
            opcode_record: [(0, 0, 0); 256],
            sload_percentile,
            total_time: 0,
            is_updated: false,
            additional_count: [0u64; CALL_OPCODE_LEN],
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

        for i in 0..CALL_OPCODE_LEN {
            self.additional_count[i] = self.additional_count[i]
                .checked_add(other.additional_count[i])
                .expect("overflow");
        }

        if !self.is_updated {
            self.opcode_record = std::mem::replace(&mut other.opcode_record, self.opcode_record);
            self.sload_percentile = other.sload_percentile;
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

        self.sload_percentile.update(&other.sload_percentile);
    }

    /// Record sload duration percentile.
    pub fn add_sload_opcode_record(&mut self, op_time_ns: f64) {
        self.sload_percentile.record(op_time_ns);
    }

    pub fn not_empty(&self) -> bool {
        self.is_updated
    }

    pub fn add_additional_count(&mut self, opcode: u8, count: u64) {
        let index = match opcode {
            // CALL
            0xF1 => 0,
            // CALLCODE
            0xF2 => 1,
            // DELEGATECALL
            0xF4 => 2,
            // STATICCALL
            0xFA => 3,
            _ => {
                println!("Add additional_count with error opcode!");
                4
            }
        };

        if index < 4 {
            self.additional_count[index] = self.additional_count[index]
                .checked_add(count)
                .expect("overflow");
        }
    }
}

/// This type represents in which function the access cache is accessed.
#[derive(Copy, Clone)]
pub enum Function {
    CodeByHash = 0,
    Storage,
    BlockHash,
    LoadCacheAccount,
}
/// This structure records the number of times cache hits/misses are accessed in each function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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

/// The additional cost (cpu cycles) incurred when CacheDb is not hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissesPenalty {
    // Record the penalty when each function hits the cache.
    pub time: AccessStats,
    /// Record the time distribution at a subtle level.
    pub percentile: TimeDistributionStats,
}

impl Default for MissesPenalty {
    fn default() -> Self {
        let percentile = TimeDistributionStats::new(US_SPAN_SIZE, NS_SPAN_SIZE);
        MissesPenalty {
            time: AccessStats::default(),
            percentile,
        }
    }
}

impl MissesPenalty {
    pub fn update(&mut self, other: &Self) {
        self.time.update(&other.time);
        self.percentile.update(&other.percentile);
    }

    fn percentile(&mut self, time_in_ns: f64) {
        self.percentile.record(time_in_ns);
    }
}

/// CacheDbRecord records the relevant information of CacheDb hits during the execution process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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

/// This structure is used to record the time consumption of each part of function
/// transact_preverified_inner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransactPreverifiedInnerTime {
    pub before_execute: u64,
    pub execute: u64,
    pub after_execute: u64,
}

impl TransactPreverifiedInnerTime {
    /// Update this struct with the other's data.
    pub fn update(&mut self, other: &Self) {
        self.before_execute = self
            .before_execute
            .checked_add(other.before_execute)
            .expect("overflow");
        self.execute = self.execute.checked_add(other.execute).expect("overflow");
        self.after_execute = self
            .after_execute
            .checked_add(other.after_execute)
            .expect("overflow");
    }
}

/// This structure is used to record the time consumption of each part of function
/// transact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TransactTime {
    /// Record total time of trasact.
    pub total: u64,
    /// Record the time consumption of function preverify_transaction_inner.
    pub preverify_transaction_inner: u64,
    /// Record the time consumption of function transact_preverified_inner.
    pub transact_preverified_inner: TransactPreverifiedInnerTime,
    /// Record the time consumption of function handler.end().
    pub handle_end: u64,
}

impl TransactTime {
    /// Update this struct with the other's data.
    pub fn update(&mut self, other: &Self) {
        self.total = self.total.checked_add(other.total).expect("overflow");
        self.preverify_transaction_inner = self
            .preverify_transaction_inner
            .checked_add(other.preverify_transaction_inner)
            .expect("overflow");
        self.transact_preverified_inner
            .update(&other.transact_preverified_inner);
        self.handle_end = self
            .handle_end
            .checked_add(other.handle_end)
            .expect("overflow");
    }
}
