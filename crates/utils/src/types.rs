use std::time::Duration;

use serde::{Deserialize, Serialize};

pub type RevmMetricRecord = OpcodeRecord;

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
            if op_time <= SLOAD_OPCODE_TIME_STEP[index] {
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

/// The number of cache hits when accessing CacheDb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheHits {
    pub hits_in_block_hash: u64,
    pub hits_in_basic: u64,
    pub hits_in_storage: u64,
    pub hits_in_code_by_hash: u64,
}

impl CacheHits {
    pub fn update(&mut self, other: &Self) {
        self.hits_in_block_hash = self
            .hits_in_block_hash
            .checked_add(other.hits_in_block_hash)
            .expect("overflow");
        self.hits_in_basic = self
            .hits_in_basic
            .checked_add(other.hits_in_basic)
            .expect("overflow");
        self.hits_in_storage = self
            .hits_in_storage
            .checked_add(other.hits_in_storage)
            .expect("overflow");
        self.hits_in_code_by_hash = self
            .hits_in_code_by_hash
            .checked_add(other.hits_in_code_by_hash)
            .expect("overflow");
    }
}

/// The number of cache misses when accessing CacheDb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheMisses {
    pub misses_in_block_hash: u64,
    pub misses_in_basic: u64,
    pub misses_in_storage: u64,
    pub misses_in_code_by_hash: u64,
}

impl CacheMisses {
    pub fn update(&mut self, other: &Self) {
        self.misses_in_block_hash = self
            .misses_in_block_hash
            .checked_add(other.misses_in_block_hash)
            .expect("overflow");
        self.misses_in_basic = self
            .misses_in_basic
            .checked_add(other.misses_in_basic)
            .expect("overflow");
        self.misses_in_storage = self
            .misses_in_storage
            .checked_add(other.misses_in_storage)
            .expect("overflow");
        self.misses_in_code_by_hash = self
            .misses_in_code_by_hash
            .checked_add(other.misses_in_code_by_hash)
            .expect("overflow");
    }
}

/// The additional cost incurred when CacheDb is not hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheMissesPenalty {
    pub penalty_in_block_hash: Duration,
    pub penalty_in_basic: Duration,
    pub penalty_in_storage: Duration,
    pub penalty_in_code_by_hash: Duration,
}
impl CacheMissesPenalty {
    pub fn update(&mut self, other: &Self) {
        self.penalty_in_block_hash = self
            .penalty_in_block_hash
            .checked_add(other.penalty_in_block_hash)
            .expect("overflow");
        self.penalty_in_basic = self
            .penalty_in_basic
            .checked_add(other.penalty_in_basic)
            .expect("overflow");
        self.penalty_in_storage = self
            .penalty_in_storage
            .checked_add(other.penalty_in_storage)
            .expect("overflow");
        self.penalty_in_code_by_hash = self
            .penalty_in_code_by_hash
            .checked_add(other.penalty_in_code_by_hash)
            .expect("overflow");
    }
}

/// CacheDbRecord records the relevant information of CacheDb hits during the execution process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheDbRecord {
    pub hits: CacheHits,
    pub misses: CacheMisses,
    pub penalty: CacheMissesPenalty,
}

impl CacheDbRecord {
    /// Update this struct with the other's data.
    pub fn update(&mut self, other: &Self) {
        self.hits.update(&other.hits);
        self.misses.update(&other.misses);
        self.penalty.update(&other.penalty);
    }

    /// The number of times CacheDb is accessed in function basic.
    pub fn total_in_basic(&self) -> u64 {
        self.hits
            .hits_in_basic
            .checked_add(self.misses.misses_in_basic)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function code_by_hash.
    pub fn total_in_code_by_hash(&self) -> u64 {
        self.hits
            .hits_in_code_by_hash
            .checked_add(self.misses.misses_in_code_by_hash)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function storage.
    pub fn total_in_storage(&self) -> u64 {
        self.hits
            .hits_in_storage
            .checked_add(self.misses.misses_in_storage)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function block_hash.
    pub fn total_in_block_hash(&self) -> u64 {
        self.hits
            .hits_in_block_hash
            .checked_add(self.misses.misses_in_block_hash)
            .expect("overflow")
    }

    /// The number of cache hits when accessing CacheDB.
    pub fn total_hits(&self) -> u64 {
        let mut total = self
            .hits
            .hits_in_basic
            .checked_add(self.hits.hits_in_code_by_hash)
            .expect("overflow");
        total = total
            .checked_add(self.hits.hits_in_storage)
            .expect("overflow");
        total = total
            .checked_add(self.hits.hits_in_block_hash)
            .expect("overflow");

        total
    }

    /// The number of cache miss when accessing CacheDB.
    pub fn total_miss(&self) -> u64 {
        let mut total = self
            .misses
            .misses_in_basic
            .checked_add(self.misses.misses_in_code_by_hash)
            .expect("overflow");
        total = total
            .checked_add(self.misses.misses_in_storage)
            .expect("overflow");
        total = total
            .checked_add(self.misses.misses_in_block_hash)
            .expect("verflow");

        total
    }

    /// The additional cost incurred when accessing CacheDb without a cache hit.
    pub fn total_penalty_times(&self) -> f64 {
        let mut total = self
            .penalty
            .penalty_in_basic
            .checked_add(self.penalty.penalty_in_code_by_hash)
            .expect("overflow");
        total = total
            .checked_add(self.penalty.penalty_in_storage)
            .expect("overflow");
        total = total
            .checked_add(self.penalty.penalty_in_block_hash)
            .expect("overflow");

        total.as_secs_f64()
    }
}
