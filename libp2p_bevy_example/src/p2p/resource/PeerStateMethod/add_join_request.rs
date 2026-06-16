use libp2p::PeerId;

use crate::p2p::resource::PeerState;

pub fn add_join_request(state: &mut PeerState, peer_id: PeerId) {
    if !state.pending_join_requests.contains(&peer_id) {
        state.pending_join_requests.push(peer_id);
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
        let mut state = PeerState::new(&Config::default(), PeerId::random());
        state.add_join_request(peer_id);
        assert!(state.pending_join_requests.contains(&peer_id));
    }
}
