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
