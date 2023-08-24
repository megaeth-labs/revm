use super::instruction::*;
use super::types::*;

#[derive(Default)]
struct Metric {
    instruction_record: InstructionMetricRecoder,
    cachedb_record: CacheDbRecord,
}

static mut METRIC_RECORDER: Option<Metric> = None;

#[ctor::ctor]
unsafe fn init() {
    METRIC_RECORDER = Some(Metric::default());
}

pub fn start_record_op() {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .start_record();
    }
}

pub fn record_op(opcode: u8) {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .record_op(opcode);
    }
}

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
pub fn get_op_record() -> OpcodeRecord {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .instruction_record
            .get_record()
    }
}

pub(super) fn hit_record() {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .cachedb_record
            .hit();
    }
}

pub(super) fn miss_record(cycles: u64) {
    unsafe {
        METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!")
            .cachedb_record
            .miss(cycles);
    }
}

/// Retrieve the records of cachedb, which will be reset after retrieval.
pub fn get_cache_record() -> CacheDbRecord {
    unsafe {
        let record = METRIC_RECORDER
            .as_mut()
            .expect("Metric recorder should not empty!");
        std::mem::replace(&mut record.cachedb_record, CacheDbRecord::default())
    }
}
