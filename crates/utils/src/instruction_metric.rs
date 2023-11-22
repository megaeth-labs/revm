use crate::time_utils::{convert_cycles_to_ms, instant::Instant};
use crate::types::*;
use std::cell::RefCell;

/// This struct is used to record information during instruction execution
/// and finally stores the data in the opcode_record field.
#[derive(Debug, Default)]
struct InstructionMetricRecoder {
    record: OpcodeRecord,
    start_time: Option<Instant>,
    pre_time: Option<Instant>,
    started: bool,
}

thread_local! {
    static INSTRUCTION_RECORDER: RefCell<InstructionMetricRecoder> = RefCell::new(InstructionMetricRecoder::default());
}

/// Start record.
pub fn start_record() {
    let now = Instant::now();

    INSTRUCTION_RECORDER.with(|recorder| {
        let mut recorder = recorder.borrow_mut();
        if !recorder.started {
            recorder.start_time = Some(now);
            recorder.pre_time = Some(now);
        }
        recorder.started = true;
    });
}

/// Record opcode execution information, recording: count, time and sload percentile.
pub fn record(opcode: u8) {
    let now = Instant::now();

    INSTRUCTION_RECORDER.with(|recorder| {
        let mut recorder = recorder.borrow_mut();

        // calculate count
        recorder.record.opcode_record[opcode as usize].0 = recorder.record.opcode_record
            [opcode as usize]
            .0
            .checked_add(1)
            .expect("overflow");

        // calculate time
        let cycles = now
            .checked_cycles_since(recorder.pre_time.expect("pre time is empty"))
            .expect("overflow");
        recorder.record.opcode_record[opcode as usize].1 = recorder.record.opcode_record
            [opcode as usize]
            .1
            .checked_add(cycles.into())
            .expect("overflow");
        recorder.pre_time = Some(now);

        // update total time
        recorder.record.total_time = now
            .checked_cycles_since(recorder.start_time.expect("start time is empty"))
            .expect("overflow")
            .into();

        // SLOAD = 0x54,
        // statistical percentile of sload duration
        if opcode == 0x54 {
            recorder
                .record
                .add_sload_opcode_record(convert_cycles_to_ms(cycles));
        }

        recorder.record.is_updated = true;
    });
}

/// Retrieve the records of opcode execution, which will be reset after retrieval.
pub fn get_record() -> OpcodeRecord {
    INSTRUCTION_RECORDER.with(|recorder| {
        let mut recorder = recorder.borrow_mut();

        recorder.start_time = None;
        recorder.pre_time = None;
        recorder.started = false;
        std::mem::replace(&mut recorder.record, OpcodeRecord::default())
    })
}

/// Record the gas consumption during opcode execution.
pub fn record_gas(opcode: u8, gas_used: u64) {
    INSTRUCTION_RECORDER.with(|recorder| {
        let mut recorder = recorder.borrow_mut();

        // calculate gas
        recorder.record.opcode_record[opcode as usize].2 = recorder.record.opcode_record
            [opcode as usize]
            .2
            .checked_add(gas_used.into())
            .expect("overflow");
    });
}
