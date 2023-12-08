//! This is a library used to support performance testing of revm.
pub mod metrics;
pub mod time_utils;

pub use metrics::Function;
pub use metrics::{HitRecord, MissRecord};
