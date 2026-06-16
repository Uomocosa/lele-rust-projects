use libp2p::PeerId;

use crate::p2p::resource::PeerState;

pub fn remove_connected_peer(state: &mut PeerState, peer_id: PeerId) {
    state.connected_peers.retain(|p| p != &peer_id);
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
        state.add_connected_peer(peer_id);
        state.remove_connected_peer(peer_id);
        assert!(!state.connected_peers.contains(&peer_id));
    }
}
