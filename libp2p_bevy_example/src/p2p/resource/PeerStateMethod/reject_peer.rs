use libp2p::PeerId;

use crate::p2p::resource::PeerState;

pub fn reject_peer(state: &mut PeerState, peer_id: PeerId) -> bool {
    if state.pending_join_requests.contains(&peer_id) {
        state.pending_join_requests.retain(|p| p != &peer_id);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::PeerState;
    use crate::p2p::Config;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let peer_id = PeerId::random();
        let config = Config::default().with_auto_accept(false);
        let mut state = PeerState::new(&config, PeerId::random());

        state.add_join_request(peer_id);
        let result = state.reject_peer(peer_id);

        assert!(result, "reject_peer should succeed");
        assert!(!state.pending_join_requests.contains(&peer_id));
        assert!(!state.connected_peers.contains(&peer_id));
    }
}
