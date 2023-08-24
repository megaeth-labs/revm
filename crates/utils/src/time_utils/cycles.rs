//! This module provides static methods that read the fine-grain CPU
//! cycle counter and translate between cycle-level times and absolute
//! time.
use std::cell::UnsafeCell;
use std::time::{Duration, Instant};

struct Cycles {
    nanos_per_cycle: UnsafeCell<f64>,
}

unsafe impl Sync for Cycles {}

static CYCLES: Cycles = Cycles {
    nanos_per_cycle: UnsafeCell::new(1.0),
};

#[ctor::ctor]
unsafe fn init() {
    let cycles_per_sec = cycles_per_sec();
    *CYCLES.nanos_per_cycle.get() = 1_000_000_000.0 / cycles_per_sec;
}

#[inline]
pub(crate) fn nanos_per_cycle() -> f64 {
    unsafe { *CYCLES.nanos_per_cycle.get() }
}

fn cycles_per_sec() -> f64 {
    // Compute the frequency of the fine-grained CPU timer: to do this,
    // take parallel time readings using both rdtsc and std::time::Instant.
    // After 10ms have elapsed, take the ratio between these readings.
    let mut old_cycles: f64 = 0.0;
    let mut cycles_per_sec: f64;

    loop {
        let (start_time, start_cycles) = (Instant::now(), rdtsc());

        loop {
            let (stop_time, stop_cycles) = (Instant::now(), rdtsc());

            let nanos = (stop_time - start_time).as_nanos();
            if nanos > 10_000_000 {
                cycles_per_sec =
                    (stop_cycles - start_cycles) as f64 * 1000_000_000.0 / nanos as f64;
                break;
            }
        }

        let delta = f64::abs(cycles_per_sec - old_cycles);
        if delta < cycles_per_sec / 100_000.0 {
            break;
        }
        old_cycles = cycles_per_sec;
    }

    cycles_per_sec
}

#[inline(always)]
pub(crate) fn rdtsc() -> u64 {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::x86::_rdtsc()
    }
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }
}

#[inline(always)]
pub fn convert_cycles_to_ns(cycles: u64) -> u64 {
    convert_cycles_to_ns_f64(cycles) as u64
}

#[inline(always)]
pub fn convert_cycles_to_ns_f64(cycles: u64) -> f64 {
    cycles as f64 * nanos_per_cycle()
}

#[inline(always)]
pub fn convert_cycles_to_ms(cycles: u64) -> u64 {
    (cycles as f64 * nanos_per_cycle() / 1_000.0) as u64
}

#[inline(always)]
pub fn convert_cycles_to_duration(cycles: u64) -> Duration {
    Duration::from_nanos(convert_cycles_to_ns(cycles))
}
