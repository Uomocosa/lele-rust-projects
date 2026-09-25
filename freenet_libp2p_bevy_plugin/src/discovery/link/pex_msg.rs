use serde::{Deserialize, Serialize};

use super::super::directory::room_payload::RoomPayload;
use super::super::gossip::peer_hint::PeerHint;
use super::super::params::room_name::RoomName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PexMsg {
    Ask,
    Resp {
        peers: Vec<PeerHint>,
        rooms: Vec<(RoomName, RoomPayload)>,
    },
}
