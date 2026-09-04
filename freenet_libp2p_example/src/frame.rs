use serde::{Deserialize, Serialize};

use crate::identity_bridge;

pub type PubkeyForFrame = identity_bridge::Pubkey;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub seq: u64,
    pub prev: u8,
    pub next: u8,
    pub author: identity_bridge::Pubkey,
    pub sig: Vec<u8>,
}

// no test_usage necessary
