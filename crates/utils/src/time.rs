use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[inline(always)]
fn rdtsc() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

pub fn get_cpu_frequency() -> Option<f64> {
    let file = File::open("/proc/cpuinfo").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        if let Some(freq_str) = line.strip_prefix("cpu MHz\t\t: ") {
            let frequency: f64 = freq_str.parse().unwrap();
            return Some(frequency * 1e6);
        }
    }

    None
}
pub fn convert_to_nanoseconds(cycles: u64, frequency: f64) -> u64 {
    let ns_per_cycle = 1_000_000_000 as f64 / frequency;
    (cycles as f64 * ns_per_cycle) as u64
}

pub fn convert_ns_to_secs(nanoseconds: u128) -> f64 {
    let seconds = nanoseconds / 1_000_000_000;
    let subsec_nanos = (nanoseconds % 1_000_000_000) as f64 / 1_000_000_000.0;
    seconds as f64 + subsec_nanos
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SpanTimeInNs {
    start: u64,
    end: u64,
}

impl SpanTimeInNs {
    pub fn to_nanoseconds(&self, frequency: f64) -> u64 {
        let cycles = self.to_cycles();
        convert_to_nanoseconds(cycles, frequency)
    }

    pub fn to_cycles(&self) -> u64 {
        self.end
            .checked_sub(self.start)
            .expect("Get spanc time error.")
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TimeRecorder {
    now: u64,
}

impl TimeRecorder {
    pub fn now() -> Self {
        TimeRecorder { now: rdtsc() }
    }

    pub fn elapsed(&mut self) -> SpanTimeInNs {
        SpanTimeInNs {
            start: self.now,
            end: rdtsc(),
        }
    }

    pub fn record_next_time(&mut self) -> SpanTimeInNs {
        let now = rdtsc();
        let span = SpanTimeInNs {
            start: self.now,
            end: now,
        };
        self.now = now;
        span
    }
}
