use libp2p::PeerId;

use crate::p2p::resource::PeerState;

pub fn accept_peer(state: &mut PeerState, peer_id: PeerId) -> bool {
    if let Some(max) = state.max_players {
        if state.connected_peers.len() >= max {
            return false;
        }
    }
    if state.pending_join_requests.contains(&peer_id) {
        state.pending_join_requests.retain(|p| p != &peer_id);
        if !state.connected_peers.contains(&peer_id) {
            state.connected_peers.push(peer_id);
        }
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
        let peer_id1 = PeerId::random();
        let peer_id2 = PeerId::random();
        let config = Config::default()
            .with_max_players(1)
            .with_auto_accept(false);
        let mut state = PeerState::new(&config, PeerId::random());

        state.add_join_request(peer_id1);
        let result1 = state.accept_peer(peer_id1);
        assert!(result1, "First peer should be accepted");
        assert!(state.connected_peers.contains(&peer_id1));

        state.add_join_request(peer_id2);
        let result2 = state.accept_peer(peer_id2);
        assert!(!result2, "Second peer should be rejected (max 1 player)");
    }
}
