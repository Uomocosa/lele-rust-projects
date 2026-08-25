use serde::{Deserialize, Serialize};

use crate::hashed_input;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputLogEntry {
    pub seq: u64,
    pub inputs: Vec<hashed_input::HashedInput>,
    pub signature: Vec<u8>,
}

// no test_usage necessary
