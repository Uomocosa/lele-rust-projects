use bevy::prelude::*;
use libp2p::PeerId;

#[derive(Resource)]
pub struct NetworkState {
    pub local_peer_id: PeerId,
    pub discovered_peers: Vec<PeerId>,
    pub connected_peers: Vec<PeerId>,
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            local_peer_id: PeerId::random(),
            discovered_peers: Vec::new(),
            connected_peers: Vec::new(),
        }
    }
}
