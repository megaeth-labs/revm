//! This module is used to measure the time consumption of various parts of function
//! transact in Revm.
use super::types::*;
use crate::time_utils::instant::Instant;

#[derive(Debug, Default)]
pub(super) struct TransactDurationRecorder {
    /// Record the starting time of function execute_and_verify_receipt.
    start_record: Instant,
    /// Record the start time of each subfunction.
    sub_record: Instant,
    /// Record the time consumption of each part of function transact.
    transact_time: TransactTime,
}

impl TransactDurationRecorder {
    /// Start record.
    pub(super) fn start_record(&mut self) {
        self.start_record = Instant::now();
    }
    /// Start sub record.
    pub(super) fn start_sub_record(&mut self) {
        self.sub_record = Instant::now();
    }

    /// Add time of preverify_transaction_inner.
    pub(super) fn preverify_transaction_inner_record(&mut self) {
        let (cycles, _) = self.record_sub_time();
        self.transact_time.preverify_transaction_inner = self
            .transact_time
            .preverify_transaction_inner
            .checked_add(cycles)
            .expect("overflow");
    }

    /// Add the time before execute opcode in transact_preverified_inner.
    pub(super) fn before_execute_record(&mut self) {
        let (cycles, _) = self.record_sub_time();
        self.transact_time.transact_preverified_inner.before_execute = self
            .transact_time
            .transact_preverified_inner
            .before_execute
            .checked_add(cycles)
            .expect("overflow");
    }

    /// Add the time of execute opcode in transact_preverified_inner.
    pub(super) fn execute_record(&mut self) {
        let (cycles, _) = self.record_sub_time();
        self.transact_time.transact_preverified_inner.execute = self
            .transact_time
            .transact_preverified_inner
            .execute
            .checked_add(cycles)
            .expect("overflow");
    }

    /// Add the time after execute opcode in transact_preverified_inner.
    pub(super) fn after_execute_record(&mut self) {
        let (cycles, _) = self.record_sub_time();
        self.transact_time.transact_preverified_inner.after_execute = self
            .transact_time
            .transact_preverified_inner
            .after_execute
            .checked_add(cycles)
            .expect("overflow");
    }

    /// Add the time of handler.end().
    pub(super) fn handler_end_record(&mut self) {
        let (cycles, now) = self.record_sub_time();
        self.transact_time.handle_end = self
            .transact_time
            .handle_end
            .checked_add(cycles)
            .expect("overflow");
        self.record_total_time(now);
    }

    /// Record total time.
    fn record_total_time(&mut self, now: Instant) {
        let cycles = now.checked_cycles_since(self.start_record).unwrap_or(0);
        self.transact_time.total = self
            .transact_time
            .total
            .checked_add(cycles)
            .expect("overflow");
    }

    /// Record time of sub function.
    fn record_sub_time(&mut self) -> (u64, Instant) {
        let now = Instant::now();
        let cycles = now.checked_cycles_since(self.sub_record).unwrap_or(0);
        self.sub_record = now;
        (cycles, now)
    }

    /// Retrieve transact time, which will be reset after retrieval.
    pub(super) fn get_transact_time(&mut self) -> TransactTime {
        std::mem::replace(&mut self.transact_time, TransactTime::default())
    }
}
