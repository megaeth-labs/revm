use serde::{Deserialize, Serialize};

pub type RevmMetricRecord = OpcodeRecord;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpcodeRecord {
    /// The abscissa is opcode type, the first element of a tuple is opcode counter, and the second element is total execute time.
    #[serde(with = "serde_arrays")]
    pub opcode_record: [(u64, std::time::Duration); 256],
    pub is_updated: bool,
}

impl Default for OpcodeRecord {
    fn default() -> Self {
        Self {
            opcode_record: [(0, std::time::Duration::default()); 256],
            is_updated: false,
        }
    }
}

impl OpcodeRecord {
    pub fn update(&mut self, other: &mut OpcodeRecord) {
        if !other.is_updated {
            return;
        }

        if !self.is_updated {
            self.opcode_record = std::mem::replace(&mut other.opcode_record, self.opcode_record);
            self.is_updated = true;
            return;
        }

        for i in 0..256 {
            self.opcode_record[i].0 = self.opcode_record[i]
                .0
                .checked_add(other.opcode_record[i].0)
                .expect("overflow");
            self.opcode_record[i].1 = self.opcode_record[i]
                .1
                .checked_add(other.opcode_record[i].1)
                .expect("overflow");
        }
    }

    pub fn not_empty(&self) -> bool {
        self.is_updated
    }
}
