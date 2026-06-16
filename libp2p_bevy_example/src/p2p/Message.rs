use libp2p::PeerId;
use serde::{Deserialize, Serialize};

use crate::p2p::PlayerInputData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    PlayerInput { tick: u64, input: PlayerInputData },
    JoinRequest { peer_id: PeerId },
    Accept { peer_id: PeerId },
    Reject { peer_id: PeerId },
    PlayerJoined { peer_id: PeerId },
    PlayerLeft { peer_id: PeerId },
    Ping { peer_id: PeerId },
    Pong { peer_id: PeerId },
}
