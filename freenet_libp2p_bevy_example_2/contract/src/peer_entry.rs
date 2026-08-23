use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEntry {
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub seq: u64,
    pub signature: Vec<u8>,
}

// no test_usage necessary
