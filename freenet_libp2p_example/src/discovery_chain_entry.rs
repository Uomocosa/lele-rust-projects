use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ChainEntry {
    pub author: [u8; 32],
    pub prev: u8,
    pub next: u8,
    pub sig: Vec<u8>,
}

// no test_usage necessary
