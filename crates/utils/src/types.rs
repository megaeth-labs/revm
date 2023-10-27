use std::time::Duration;

use serde::{Deserialize, Serialize};

pub type RevmMetricRecord = OpcodeRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcodeRecord {
    /// The abscissa is opcode type, tuple means: (opcode counter, time, gas).
    #[serde(with = "serde_arrays")]
    pub opcode_record: [(u64, Duration, i128); 256],
    pub total_time: Duration,
    pub is_updated: bool,
}

impl Default for OpcodeRecord {
    fn default() -> Self {
        Self {
            opcode_record: [(0, Duration::default(), 0); 256],
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
