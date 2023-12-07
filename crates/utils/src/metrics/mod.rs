mod cachedb;
mod instruction;
mod metric;
pub mod types;

pub use cachedb::{HitRecord, MissRecord};
pub use metric::{get_cache_record, get_op_record, record_gas, record_op, start_record_op};
pub use types::Function;
