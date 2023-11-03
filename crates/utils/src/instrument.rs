use crate::types::*;
use minstant::Instant;
use std::cell::RefCell;

#[derive(Debug, Default)]
struct Instrument {
    record: OpcodeRecord,
    start_time: Option<Instant>,
    pre_time: Option<Instant>,
    started: bool,
}

thread_local! {
    static INSTANCE: RefCell<Instrument> = RefCell::new(Instrument::default());
}

pub fn start_record() {
    let now = minstant::Instant::now();

    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();
        instance.start_time = Some(now);
        if !instance.started {
            instance.pre_time = Some(now);
        }
        instance.started = true;
    });
}

pub fn record(opcode: u8, gas_used: u64, gas_refund: i64) {
    let now = minstant::Instant::now();
    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();

        // calculate count
        instance.record.opcode_record[opcode as usize].0 = instance.record.opcode_record
            [opcode as usize]
            .0
            .checked_add(1)
            .expect("overflow");

        // calculate time
        let duration = now
            .checked_duration_since(instance.pre_time.expect("pre time is empty"))
            .expect("overflow");
        instance.record.opcode_record[opcode as usize].1 = instance.record.opcode_record
            [opcode as usize]
            .1
            .checked_add(duration)
            .expect("overflow");
        instance.pre_time = Some(now);

        // SLOAD = 0x54,
        // statistical percentile of sload duration
        if opcode == 0x54 {
            instance
                .record
                .add_sload_opcode_record(duration.as_micros());
        }

        // calculate gas
        instance.record.opcode_record[opcode as usize].2 = instance.record.opcode_record
            [opcode as usize]
            .2
            .checked_add(gas_used.into())
            .expect("overflow");

        if gas_refund != 0 {
            instance.record.opcode_record[opcode as usize].2 = instance.record.opcode_record
                [opcode as usize]
                .2
                .checked_sub(gas_refund.into())
                .expect("overflow");
        }

        instance.record.is_updated = true;
    });
}

pub fn end_record() {
    let now = minstant::Instant::now();

    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();
        instance.record.total_time = now
            .checked_duration_since(instance.start_time.expect("start time is empty"))
            .expect("overflow");
    });
}

pub fn get_record() -> OpcodeRecord {
    INSTANCE.with(|instance| {
        let mut instance = instance.borrow_mut();

        instance.start_time = None;
        instance.pre_time = None;
        instance.started = false;
        std::mem::replace(&mut instance.record, OpcodeRecord::default())
    })
}
