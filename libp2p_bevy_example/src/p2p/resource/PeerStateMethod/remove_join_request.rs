use libp2p::PeerId;

use crate::p2p::resource::PeerState;

pub fn remove_join_request(state: &mut PeerState, peer_id: PeerId) {
    state.pending_join_requests.retain(|p| p != &peer_id);
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::PeerState;
    use crate::p2p::Config;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let peer_id = PeerId::random();
        let mut state = PeerState::new(&Config::default(), PeerId::random());
        state.add_join_request(peer_id);
        state.remove_join_request(peer_id);
        assert!(!state.pending_join_requests.contains(&peer_id));
    }
}
