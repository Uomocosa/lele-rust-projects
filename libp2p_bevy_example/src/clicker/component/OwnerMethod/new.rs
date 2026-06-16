use crate::clicker::component::Owner;
use libp2p::PeerId;

pub fn new(peer_id: PeerId) -> Owner {
    Owner { peer_id }
}

#[cfg(test)]
mod tests {
    use crate::clicker::component::Owner;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let peer_id = PeerId::random();
        let owner = Owner::new(peer_id);
        assert_eq!(owner.peer_id, peer_id);
    }
}
