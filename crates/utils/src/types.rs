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

    pub fn update(&mut self, other: &Self) {
        self.step = self.step.checked_add(other.step).expect("overflow");
        self.step_end = self.step_end.checked_add(other.step_end).expect("overflow");
        self.env = self.env.checked_add(other.env).expect("overflow");
        self.load_account = self
            .load_account
            .checked_add(other.load_account)
            .expect("overflow");
        self.block_hash = self
            .block_hash
            .checked_add(other.block_hash)
            .expect("overflow");
        self.balance = self.balance.checked_add(other.balance).expect("overflow");
        self.code = self.code.checked_add(other.code).expect("overflow");
        self.code_hash = self
            .code_hash
            .checked_add(other.code_hash)
            .expect("overflow");
        self.sload = self.sload.checked_add(other.sload).expect("overflow");
        self.sstore = self.sstore.checked_add(other.sstore).expect("overflow");
        self.log = self.log.checked_add(other.log).expect("overflow");
        self.selfdestruct = self
            .selfdestruct
            .checked_add(other.selfdestruct)
            .expect("overflow");
        self.create = self.create.checked_add(other.create).expect("overflow");
        self.call = self.call.checked_add(other.call).expect("overflow");
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    pub fn update(&mut self, other: &mut RevmMetricRecord) {
        if self.opcode_time.is_none() {
            self.opcode_time = other.opcode_time.take();
        } else {
            for (key, value) in other.opcode_time.take().unwrap() {
                self.opcode_time
                    .as_mut()
                    .expect("None")
                    .entry(key)
                    .and_modify(|(v1, v2)| {
                        v1.checked_add(value.0).expect("overflow");
                        v2.checked_add(value.1).expect("overflow");
                    })
                    .or_insert(value);
            }
        }

        self.host_time.update(&other.host_time);

        self.cache_hits.0 = self
            .cache_hits
            .0
            .checked_add(other.cache_hits.0)
            .expect("overflow");
        self.cache_hits.1 = self
            .cache_hits
            .1
            .checked_add(other.cache_hits.1)
            .expect("overflow");
        self.cache_hits.2 = self
            .cache_hits
            .2
            .checked_add(other.cache_hits.2)
            .expect("overflow");
        self.cache_hits.3 = self
            .cache_hits
            .3
            .checked_add(other.cache_hits.3)
            .expect("overflow");

        self.cache_misses.0 = self
            .cache_misses
            .0
            .checked_add(other.cache_misses.0)
            .expect("overflow");
        self.cache_misses.1 = self
            .cache_misses
            .1
            .checked_add(other.cache_misses.1)
            .expect("overflow");
        self.cache_misses.2 = self
            .cache_misses
            .2
            .checked_add(other.cache_misses.2)
            .expect("overflow");
        self.cache_misses.3 = self
            .cache_misses
            .3
            .checked_add(other.cache_misses.3)
            .expect("overflow");

        self.cache_misses_penalty.0 = self
            .cache_misses_penalty
            .0
            .checked_add(other.cache_misses_penalty.0)
            .expect("overflow");
        self.cache_misses_penalty.1 = self
            .cache_misses_penalty
            .1
            .checked_add(other.cache_misses_penalty.1)
            .expect("overflow");
        self.cache_misses_penalty.2 = self
            .cache_misses_penalty
            .2
            .checked_add(other.cache_misses_penalty.2)
            .expect("overflow");
        self.cache_misses_penalty.3 = self
            .cache_misses_penalty
            .3
            .checked_add(other.cache_misses_penalty.3)
            .expect("overflow");
    }
}
