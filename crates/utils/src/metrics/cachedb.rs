use super::metric::*;
use crate::time_utils::instant::Instant;

pub struct HitRecord;

impl HitRecord {
    pub fn new() -> HitRecord {
        HitRecord
    }
}

impl Drop for HitRecord {
    fn drop(&mut self) {
        hit_record();
    }
}

pub struct MissRecord {
    start_time: Instant,
}

impl MissRecord {
    pub fn new() -> MissRecord {
        MissRecord {
            start_time: Instant::now(),
        }
    }
}

impl Drop for MissRecord {
    fn drop(&mut self) {
        let now = Instant::now();
        let cycles = now.checked_cycles_since(self.start_time).expect("overflow");

        miss_record(cycles);
    }
}
