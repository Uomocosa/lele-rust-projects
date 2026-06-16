use bevy::prelude::Resource;
use libp2p::PeerId;

use crate::p2p::resource::PeerStateMethod;
use crate::p2p::Config;

#[derive(Resource)]
pub struct PeerState {
    pub max_players: Option<usize>,
    pub local_peer_id: PeerId,
    pub connected_peers: Vec<PeerId>,
    pub discovered_peers: Vec<PeerId>,
    pub pending_join_requests: Vec<PeerId>,
}

#[rustfmt::skip]
impl PeerState {
    pub fn new(config: &Config, local_peer_id: PeerId) -> Self { PeerStateMethod::new(config, local_peer_id) }
    pub fn add_discovered_peer(&mut self, peer_id: PeerId) { PeerStateMethod::add_discovered_peer(self, peer_id) }
    pub fn add_connected_peer(&mut self, peer_id: PeerId) { PeerStateMethod::add_connected_peer(self, peer_id) }
    pub fn remove_connected_peer(&mut self, peer_id: PeerId) { PeerStateMethod::remove_connected_peer(self, peer_id) }
    pub fn add_join_request(&mut self, peer_id: PeerId) { PeerStateMethod::add_join_request(self, peer_id) }
    pub fn remove_join_request(&mut self, peer_id: PeerId) { PeerStateMethod::remove_join_request(self, peer_id) }
    pub fn accept_peer(&mut self, peer_id: PeerId) -> bool { PeerStateMethod::accept_peer(self, peer_id) }
    pub fn reject_peer(&mut self, peer_id: PeerId) -> bool { PeerStateMethod::reject_peer(self, peer_id) }
}
