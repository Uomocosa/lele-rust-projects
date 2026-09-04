use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct PeerRecord {
    pub peer_id: Vec<u8>,
    pub addrs: Vec<String>,
}

// no test_usage necessary
