use serde::{Deserialize, Serialize};

use super::super::directory::Entry;
use super::super::gossip::peer_hint::PeerHint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PexMsg {
    Ask,
    Resp {
        peers: Vec<PeerHint>,
        rooms: Vec<(String, Entry)>,
    },
}
