use crate::time_utils::{convert_cycles_to_ns_f64, instant::Instant};
use crate::types::*;
use std::cell::RefCell;

thread_local! {
    static CACHEDB_RECORDER: RefCell<CacheDbRecord> = RefCell::new(CacheDbRecord::default());
}

fn hit_record(function: FunctionType) {
    CACHEDB_RECORDER.with(|recorder| {
        let mut recorder = recorder.borrow_mut();

        match function {
            FunctionType::Basic => {
                recorder.hits.basic = recorder.hits.basic.checked_add(1).expect("overflow")
            }
            FunctionType::CodeByHash => {
                recorder.hits.code_by_hash =
                    recorder.hits.code_by_hash.checked_add(1).expect("overflow")
            }
            FunctionType::Storage => {
                recorder.hits.storage = recorder.hits.storage.checked_add(1).expect("overflow")
            }
            FunctionType::BlockHash => {
                recorder.hits.block_hash =
                    recorder.hits.block_hash.checked_add(1).expect("overflow")
            }
            FunctionType::LoadAccount => {
                recorder.hits.load_account =
                    recorder.hits.load_account.checked_add(1).expect("overflow")
            }
        }
    });
}

fn miss_record(function: FunctionType, cycles: u64) {
    CACHEDB_RECORDER.with(|recorder| {
        let mut recorder = recorder.borrow_mut();
        match function {
            FunctionType::Basic => {
                recorder.misses.basic = recorder.misses.basic.checked_add(1).expect("overflow");
                recorder.penalty.basic = recorder
                    .penalty
                    .basic
                    .checked_add(cycles)
                    .expect("overflow");
            }
            FunctionType::CodeByHash => {
                recorder.misses.code_by_hash = recorder
                    .misses
                    .code_by_hash
                    .checked_add(1)
                    .expect("overflow");
                recorder.penalty.code_by_hash = recorder
                    .penalty
                    .code_by_hash
                    .checked_add(cycles)
                    .expect("overflow");
            }
            FunctionType::Storage => {
                recorder.misses.storage = recorder.misses.storage.checked_add(1).expect("overflow");
                recorder.penalty.storage = recorder
                    .penalty
                    .storage
                    .checked_add(cycles)
                    .expect("overflow");
            }
            FunctionType::BlockHash => {
                recorder.misses.block_hash =
                    recorder.misses.block_hash.checked_add(1).expect("overflow");
                recorder.penalty.block_hash = recorder
                    .penalty
                    .block_hash
                    .checked_add(cycles)
                    .expect("overflow");
            }
            FunctionType::LoadAccount => {
                recorder.misses.load_account = recorder
                    .misses
                    .load_account
                    .checked_add(1)
                    .expect("overflow");
                recorder.penalty.load_account = recorder
                    .penalty
                    .load_account
                    .checked_add(cycles)
                    .expect("overflow");
            }
        }

        recorder
            .penalty
            .percentile(convert_cycles_to_ns_f64(cycles));
    });
}

/// Retrieve the records of cachedb, which will be reset after retrieval.
pub fn get_record() -> CacheDbRecord {
    CACHEDB_RECORDER.with(|recorder| {
        let mut record = recorder.borrow_mut();
        std::mem::replace(&mut *record, CacheDbRecord::default())
    })
}

/// This type represents in which function the access cache is accessed.
#[derive(Copy, Clone)]
pub enum FunctionType {
    Basic,
    CodeByHash,
    Storage,
    BlockHash,
    LoadAccount,
}

pub struct HitRecord {
    function: FunctionType,
}

impl HitRecord {
    pub fn new(function: FunctionType) -> HitRecord {
        HitRecord { function }
    }
}

impl Drop for HitRecord {
    fn drop(&mut self) {
        hit_record(self.function);
    }
}

pub struct MissRecord {
    function: FunctionType,
    start_time: Instant,
}

impl MissRecord {
    pub fn new(function: FunctionType) -> MissRecord {
        MissRecord {
            function,
            start_time: Instant::now(),
        }
    }
}

impl Drop for MissRecord {
    fn drop(&mut self) {
        let now = Instant::now();
        let cycles = now.checked_cycles_since(self.start_time).expect("overflow");

        miss_record(self.function, cycles);
    }
}
