use crate::{bytes::Bytes, B160, B256};
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
    pub address: B160,
    pub topics: Vec<B256>,
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    pub data: Bytes,
}

#[cfg(feature = "enable_opcode_metrics")]
impl Log {
    pub fn size(&self) -> usize {
        self.topics.len() * 32 + std::mem::size_of::<Log>()
    }
}
