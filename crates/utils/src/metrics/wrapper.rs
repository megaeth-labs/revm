//! This module encapsulates functions to support recording metrics in a RAII manner.
use super::metric::*;
use super::types::*;
use crate::time_utils::instant::Instant;

pub struct HitRecord {
    function: Function,
}

impl HitRecord {
    pub fn new(function: Function) -> HitRecord {
        HitRecord { function }
    }
}

impl Drop for HitRecord {
    fn drop(&mut self) {
        hit_record(self.function);
    }
}

pub struct MissRecord {
    function: Function,
    start_time: Instant,
}

impl MissRecord {
    pub fn new(function: Function) -> MissRecord {
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

pub struct PreverifyTransactionInnerRecord;

impl PreverifyTransactionInnerRecord {
    pub fn new() -> Self {
        transact_sub_record();
        Self
    }
}

impl Drop for PreverifyTransactionInnerRecord {
    fn drop(&mut self) {
        preverify_transaction_inner_record();
    }
}

pub struct HandlerEndRecord;

impl HandlerEndRecord {
    pub fn new() -> Self {
        transact_sub_record();
        Self
    }
}

impl Drop for HandlerEndRecord {
    fn drop(&mut self) {
        handler_end_record();
    }
}

pub struct ExecuteEndRecord;

impl ExecuteEndRecord {
    pub fn new() -> Self {
        execute_record();
        Self
    }
}

impl Drop for ExecuteEndRecord {
    fn drop(&mut self) {
        after_execute_record();
    }
}

pub struct OpcodeExecuteRecord(u8);

impl OpcodeExecuteRecord {
    pub fn new(opcode: u8) -> Self {
        record_before_op(opcode);
        OpcodeExecuteRecord(opcode)
    }
}

impl Drop for OpcodeExecuteRecord {
    fn drop(&mut self) {
        record_op(self.0)
    }
}
