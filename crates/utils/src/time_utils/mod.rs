//! Provide some time measurement related functions and types.
mod cycles;
pub mod instant;

pub use cycles::{
    convert_cycles_to_duration, convert_cycles_to_ms, convert_cycles_to_ns,
    convert_cycles_to_ns_f64,
};
