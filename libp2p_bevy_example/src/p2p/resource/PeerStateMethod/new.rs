use libp2p::PeerId;

use crate::p2p::resource::PeerState;
use crate::p2p::Config;

pub fn new(config: &Config, local_peer_id: PeerId) -> PeerState {
    PeerState {
        max_players: config.max_players,
        local_peer_id,
        connected_peers: Vec::new(),
        discovered_peers: Vec::new(),
        pending_join_requests: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::PeerState;
    use crate::p2p::Config;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let peer_id = PeerId::random();
        let state = PeerState::new(&Config::default(), peer_id);
        assert_eq!(state.local_peer_id, peer_id);
        assert!(state.connected_peers.is_empty());
        assert!(state.discovered_peers.is_empty());
    }
}
