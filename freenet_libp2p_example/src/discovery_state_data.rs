use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::discovery_chain_entry;
use crate::discovery_peer_record;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct StateData {
    pub peers: BTreeMap<[u8; 32], discovery_peer_record::PeerRecord>,
    pub chain: BTreeMap<u64, discovery_chain_entry::ChainEntry>,
    pub sigs: BTreeMap<[u8; 32], Vec<u8>>,
}

// no test_usage necessary
