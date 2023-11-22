//! This module defines some types used for revm metrics.
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

/// The number of cache hits when accessing CacheDb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheHits {
    pub block_hash: u64,
    pub basic: u64,
    pub storage: u64,
    pub code_by_hash: u64,
    pub load_account: u64,
}

impl CacheHits {
    pub fn update(&mut self, other: &Self) {
        self.block_hash = self
            .block_hash
            .checked_add(other.block_hash)
            .expect("overflow");
        self.basic = self.basic.checked_add(other.basic).expect("overflow");
        self.storage = self.storage.checked_add(other.storage).expect("overflow");
        self.code_by_hash = self
            .code_by_hash
            .checked_add(other.code_by_hash)
            .expect("overflow");
        self.load_account = self
            .load_account
            .checked_add(other.load_account)
            .expect("overflow");
    }
}

/// The number of cache misses when accessing CacheDb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy, Default)]
pub struct CacheMisses {
    pub block_hash: u64,
    pub basic: u64,
    pub storage: u64,
    pub code_by_hash: u64,
    pub load_account: u64,
}

impl CacheMisses {
    pub fn update(&mut self, other: &Self) {
        self.block_hash = self
            .block_hash
            .checked_add(other.block_hash)
            .expect("overflow");
        self.basic = self.basic.checked_add(other.basic).expect("overflow");
        self.storage = self.storage.checked_add(other.storage).expect("overflow");
        self.code_by_hash = self
            .code_by_hash
            .checked_add(other.code_by_hash)
            .expect("overflow");
        self.load_account = self
            .load_account
            .checked_add(other.load_account)
            .expect("overflow");
    }
}

const US_PENALTY_STEP_SIZE: usize = 200;
const NS_PENALTY_STEP_SIZE: usize = 10;
/// The additional cost (cpu cycles) incurred when CacheDb is not hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub struct CacheMissesPenalty {
    pub block_hash: u64,
    pub basic: u64,
    pub storage: u64,
    pub code_by_hash: u64,
    pub load_account: u64,
    #[serde(with = "serde_arrays")]
    pub us_percentile: [u64; US_PENALTY_STEP_SIZE],
    #[serde(with = "serde_arrays")]
    pub ns_percentile: [u64; NS_PENALTY_STEP_SIZE],
}

impl Default for CacheMissesPenalty {
    fn default() -> Self {
        CacheMissesPenalty {
            block_hash: 0,
            basic: 0,
            storage: 0,
            code_by_hash: 0,
            load_account: 0,
            us_percentile: [0; US_PENALTY_STEP_SIZE],
            ns_percentile: [0; NS_PENALTY_STEP_SIZE],
        }
    }
}

impl CacheMissesPenalty {
    pub fn update(&mut self, other: &Self) {
        self.block_hash = self
            .block_hash
            .checked_add(other.block_hash)
            .expect("overflow");
        self.basic = self.basic.checked_add(other.basic).expect("overflow");
        self.storage = self.storage.checked_add(other.storage).expect("overflow");
        self.code_by_hash = self
            .code_by_hash
            .checked_add(other.code_by_hash)
            .expect("overflow");
        self.load_account = self
            .load_account
            .checked_add(other.load_account)
            .expect("overflow");

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
            .basic
            .checked_add(self.misses.basic)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function code_by_hash.
    pub fn total_in_code_by_hash(&self) -> u64 {
        self.hits
            .code_by_hash
            .checked_add(self.misses.code_by_hash)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function storage.
    pub fn total_in_storage(&self) -> u64 {
        self.hits
            .storage
            .checked_add(self.misses.storage)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function block_hash.
    pub fn total_in_block_hash(&self) -> u64 {
        self.hits
            .block_hash
            .checked_add(self.misses.block_hash)
            .expect("overflow")
    }

    /// The number of times CacheDb is accessed in function load_account.
    pub fn total_in_load_account(&self) -> u64 {
        self.hits
            .load_account
            .checked_add(self.misses.load_account)
            .expect("overflow")
    }

    /// The number of cache hits when accessing CacheDB.
    pub fn total_hits(&self) -> u64 {
        let mut total = self
            .hits
            .basic
            .checked_add(self.hits.code_by_hash)
            .expect("overflow");
        total = total.checked_add(self.hits.storage).expect("overflow");
        total = total.checked_add(self.hits.block_hash).expect("overflow");
        total = total.checked_add(self.hits.load_account).expect("overflow");

        total
    }

    /// The number of cache miss when accessing CacheDB.
    pub fn total_miss(&self) -> u64 {
        let mut total = self
            .misses
            .basic
            .checked_add(self.misses.code_by_hash)
            .expect("overflow");
        total = total.checked_add(self.misses.storage).expect("overflow");
        total = total.checked_add(self.misses.block_hash).expect("verflow");
        total = total
            .checked_add(self.misses.load_account)
            .expect("overflow");

        total
    }

    /// The additional cost incurred when accessing CacheDb without a cache hit.
    pub fn total_penalty_times(&self) -> u64 {
        let mut total = self
            .penalty
            .basic
            .checked_add(self.penalty.code_by_hash)
            .expect("overflow");
        total = total.checked_add(self.penalty.storage).expect("overflow");
        total = total
            .checked_add(self.penalty.block_hash)
            .expect("overflow");
        total = total
            .checked_add(self.penalty.load_account)
            .expect("overflow");

        total
    }
}
