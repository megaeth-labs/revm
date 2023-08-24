use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default, Copy)]
pub struct HostTime {
    pub step: u128,
    pub step_end: u128,
    pub env: u128,
    pub load_account: u128,
    pub block_hash: u128,
    pub balance: u128,
    pub code: u128,
    pub code_hash: u128,
    pub sload: u128,
    pub sstore: u128,
    pub log: u128,
    pub selfdestruct: u128,
    pub create: u128,
    pub call: u128,
}

impl HostTime {
    pub fn not_empty(&self) -> bool {
        self.step != 0
            || self.step_end != 0
            || self.env != 0
            || self.load_account != 0
            || self.block_hash != 0
            || self.balance != 0
            || self.code != 0
            || self.code_hash != 0
            || self.sload != 0
            || self.sstore != 0
            || self.log != 0
            || self.selfdestruct != 0
            || self.create != 0
            || self.call != 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RevmMetricRecord {
    /// Opcode time: key: Opcode, value: (opcode_counter, total_execute_time).
    pub opcode_time: Option<HashMap<u8, (u64, u64)>>,
    /// Total host time.
    pub host_time: HostTime,
    /// cache_hits: (hit_in_basic, hit_in_code_by_hash, hit_in_storage, hit_in_block_hash).
    pub cache_hits: (u64, u64, u64, u64),
    /// cache_misses: (misses_in_basic, misses_in_code_by_hash, misses_in_storage, misses_in_block_hash).
    pub cache_misses: (u64, u64, u64, u64),
    /// cache_misses_penalty: (penalty_in_basic, penalty_in_code_by_hash, penalty_in_storage, penalty_in_block_hash).
    pub cache_misses_penalty: (u128, u128, u128, u128),
}

impl RevmMetricRecord {
    pub fn not_empty(&self) -> bool {
        if !self.opcode_time.is_none()
            || self.host_time.not_empty()
            || self.cache_misses_penalty != (0, 0, 0, 0)
            || self.cache_hits != (0, 0, 0, 0)
            || self.cache_misses != (0, 0, 0, 0)
        {
            return true;
        }
        false
    }
}
