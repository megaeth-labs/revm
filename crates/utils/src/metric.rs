use crate::time_utils::{convert_cycles_to_ms, instant::Instant};
use crate::types::*;
use std::cell::RefCell;

/// This struct is used to record information during opcode execution
/// and finally stores the data in the opcode_record field.
#[derive(Debug, Default)]
struct MetricRecoder {
    record: OpcodeRecord,
    start_time: Option<Instant>,
    pre_time: Option<Instant>,
    started: bool,
}

thread_local! {
    static INSTANCE: RefCell<MetricRecoder> = RefCell::new(MetricRecoder::default());
}

/// Start record.
pub fn start_record() {
    let now = Instant::now();

    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();
        if !instance.started {
            instance.start_time = Some(now);
            instance.pre_time = Some(now);
        }
        instance.started = true;
    });
}

/// Record opcode execution information, recording: count, time and sload percentile.
pub fn record(opcode: u8) {
    let now = Instant::now();

    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();

        // calculate count
        instance.record.opcode_record[opcode as usize].0 = instance.record.opcode_record
            [opcode as usize]
            .0
            .checked_add(1)
            .expect("overflow");

        // calculate time
        let cycles = now
            .checked_cycles_since(instance.pre_time.expect("pre time is empty"))
            .expect("overflow");
        instance.record.opcode_record[opcode as usize].1 = instance.record.opcode_record
            [opcode as usize]
            .1
            .checked_add(cycles.into())
            .expect("overflow");
        instance.pre_time = Some(now);

        // update total time
        instance.record.total_time = now
            .checked_cycles_since(instance.start_time.expect("start time is empty"))
            .expect("overflow")
            .into();

        // SLOAD = 0x54,
        // statistical percentile of sload duration
        if opcode == 0x54 {
            instance
                .record
                .add_sload_opcode_record(convert_cycles_to_ms(cycles));
        }

        instance.record.is_updated = true;
    });
}

/// Retrieve the records of opcode execution, which will be reset after retrieval.
pub fn get_record() -> OpcodeRecord {
    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();

        instance.start_time = None;
        instance.pre_time = None;
        instance.started = false;
        std::mem::replace(&mut instance.record, OpcodeRecord::default())
    })
}

/// Record the gas consumption during opcode execution.
pub fn record_gas(opcode: u8, gas_used: u64) {
    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();

        // calculate gas
        instance.record.opcode_record[opcode as usize].2 = instance.record.opcode_record
            [opcode as usize]
            .2
            .checked_add(gas_used.into())
            .expect("overflow");
    });
}
