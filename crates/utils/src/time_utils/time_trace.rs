//! This time trace can only be used in single threads.
use super::cycles;

static mut TIME_RECORDER: TimeTrace = TimeTrace {
    buffer: INIT_BUFFER,
};

/// Determines the number of events we can retain as an exponent of 2
const BUFFER_SIZE_EXP: u8 = 20;
/// Total number of events that we can retain any given time.
const BUFFER_SIZE: usize = 1 << BUFFER_SIZE_EXP;
/// Bit mask used to implement a circular event buffer
const BUFFER_MASK: usize = BUFFER_SIZE - 1;

/// Identify whether to save historical data.
const KEEP_OLD_EVENTS: bool = false;

const INIT_EVENT: Event = Event {
    timestamp: 0,
    format: None,
};

const INIT_BUFFER: Buffer = Buffer {
    events: [INIT_EVENT; BUFFER_SIZE],
    next_index: 0,
    has_record: false,
};

/// This structure holds one entry in the TimeTrace.
#[derive(Default, Clone)]
struct Event {
    /// Time when a particular event occurred.
    timestamp: u64,
    /// Format string describing the event.
    format: Option<String>,
}

/// Represents a sequence of events. Has a fixed capacity, so slots are re-used on a
/// circular basis.  
struct Buffer {
    /// Holds information from the most recent calls to the record method.
    events: [Event; BUFFER_SIZE],
    /// Index within events of the slot to use for the next call to the record method.
    next_index: usize,
    /// Indicating that there is a record.
    has_record: bool,
}

impl Buffer {
    /// Record an event in the buffer.
    fn record(&mut self, timestamp: u64, format: String) {
        self.events[self.next_index].timestamp = timestamp;
        self.events[self.next_index].format = Some(format);
        self.next_index = (self.next_index + 1) & BUFFER_MASK;
        self.has_record = true;
        // println!("index: {:?}, format: {:?}", self.next_index-1, self.events[self.next_index-1].format);
    }

    /// Print all existing trace records to stdout.
    fn print(&self) {
        if self.has_record {
            self.print_internal();
        }
    }

    fn print_internal(&self) {
        // Find the first (oldest) event in trace. This will be events[0] if we
        // never completely filled the buffer, otherwise events[nextIndex+1].
        let index = (self.next_index + 1) % BUFFER_SIZE;
        let mut current = if self.events[index].format.is_some() {
            index
        } else {
            0
        };

        // Get the starting time.
        let start_time = if !KEEP_OLD_EVENTS {
            let mut time = 0;
            if self.events[current].format.is_some() && self.events[current].timestamp > time {
                time = self.events[current].timestamp;
            }
            time
        } else {
            let mut time = u64::MAX;
            if self.events[current].format.is_some() && self.events[current].timestamp < time {
                time = self.events[current].timestamp;
            }
            time
        };

        // Skip all events before the starting time.
        loop {
            if self.events[current].format.is_some()
                && self.events[current].timestamp < start_time as u64
                && current != self.next_index
            {
                current = (current + 1) % BUFFER_SIZE;
            } else {
                break;
            }
        }

        // Print all events
        let mut pre_time = 0.0;
        let mut printed_anything = false;
        loop {
            if !printed_anything {
                println!("CYCLES_PER_SECOND {:?}", cycles::per_sec());
                println!("START_CYCLES {:?}", start_time);

                printed_anything = true;
            }

            if self.events[current].timestamp < start_time {
                break;
            }

            let cycles = self.events[current].timestamp - start_time;
            let ns = cycles::convert_cycles_to_ns_f64(cycles);
            println!(
                "{:13.3} ns | (+{:10.3} ns): {}",
                ns,
                ns - pre_time,
                self.events[current].format.clone().unwrap_or_default(),
            );
            pre_time = ns;

            current = (current + 1) & BUFFER_MASK;
        }
    }
}

struct TimeTrace {
    buffer: Buffer,
}

impl TimeTrace {
    fn record(&mut self, format: String) {
        self.buffer.record(cycles::rdtsc(), format);
    }

    fn print(&self) {
        self.buffer.print();
    }
}

pub fn record(format: String) {
    unsafe {
        TIME_RECORDER.record(format);
    }
}

pub fn trace_print() {
    unsafe {
        TIME_RECORDER.print();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_record_and_print() {
        record("1".to_string());
        std::thread::sleep(Duration::from_nanos(1_000_000_000));
        record("2".to_string());
        std::thread::sleep(Duration::from_nanos(2_000_000_000));
        record("3".to_string());
        trace_print();
    }
}
