use libp2p::PeerId;

use crate::p2p::resource::PeerState;

pub fn add_discovered_peer(state: &mut PeerState, peer_id: PeerId) {
    if !state.discovered_peers.contains(&peer_id) {
        state.discovered_peers.push(peer_id);
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
        state.add_discovered_peer(peer_id);
        assert!(state.discovered_peers.contains(&peer_id));
    }
}
