use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Params {
    pub namespace: [u8; 32],
    pub max_members: u16,
}

// no test_usage necessary
