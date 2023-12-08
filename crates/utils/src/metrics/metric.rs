//! In this module, a large structure is used to record all the measurement
//! metrics of Revm, while providing some functions for measuring metrics
//! in the source code and some functions for obtaining the final metrics
//! externally.
use super::instruction::*;
use super::types::*;

/// This structure records all metric information for measuring Revm.
#[derive(Default)]
struct Metric {
    /// Recording instruction metrics.
    instruction_record: InstructionMetricRecoder,
    /// Recording cache metrics.
    cachedb_record: CacheDbRecord,
}

static mut METRIC_RECORDER: Option<Metric> = None;

// This function will be called directly during program initialization.
#[ctor::ctor]
unsafe fn init() {
    METRIC_RECORDER = Some(Metric::default());
}

/// Start to record the information of opcode execution, which will be called
/// in the source code.
pub fn start_record_op() {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .start_record();
    }
}

/// Record the information of opcode execution, which will be called in the
/// source code.
pub fn record_op(opcode: u8) {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .record_op(opcode);
    }
}

/// Record the gas of opcode execution, which will be called in the source code.
pub fn record_gas(opcode: u8, gas_used: u64) {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .record_gas(opcode, gas_used);
    }
}

/// Retrieve the records of opcode execution, which will be reset after retrieval.
/// It will be called by the code of reth.
pub fn get_op_record() -> OpcodeRecord {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .get_record()
    }
}

/// The function called upon cache hit, which is encapsulated in HitRecord.
pub(super) fn hit_record(function: Function) {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .cachedb_record
            .hit(function);
    }
}

/// The function called upon cache miss, which is encapsulated in MissRecord.
pub(super) fn miss_record(function: Function, cycles: u64) {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .cachedb_record
            .miss(function, cycles);
    }
}

/// Retrieve the records of cachedb, which will be reset after retrieval.
/// It will be called by the code of reth.
pub fn get_cache_record() -> CacheDbRecord {
    unsafe {
        let record = METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!");
        std::mem::replace(&mut record.cachedb_record, CacheDbRecord::default())
    }
}
