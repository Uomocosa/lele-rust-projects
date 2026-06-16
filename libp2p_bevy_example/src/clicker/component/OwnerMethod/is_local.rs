use crate::clicker::component::Owner;
use crate::p2p::resource::PeerState;

pub fn is_local(owner: &Owner, p2p_state: &PeerState) -> bool {
    owner.peer_id == p2p_state.local_peer_id
}

#[cfg(test)]
mod tests {
    use crate::clicker::component::Owner;
    use crate::p2p::resource::PeerState;
    use crate::p2p::Config;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let peer_id = PeerId::random();
        let owner = Owner::new(peer_id);
        let state = PeerState::new(&Config::default(), peer_id);
        assert!(super::is_local(&owner, &state));
    }
}
