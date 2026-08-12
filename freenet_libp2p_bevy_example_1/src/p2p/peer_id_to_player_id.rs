use libp2p::PeerId;

use crate::boxes;

pub fn peer_id_to_player_id(peer_id: &PeerId) -> boxes::PlayerId {
    let bytes = peer_id.to_bytes();
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&bytes[..8]);
    boxes::PlayerId(u64::from_be_bytes(id_bytes))
}

#[cfg(test)]
mod tests {
    use super::peer_id_to_player_id;

    #[test]
    fn test_usage() {
        let peer_id = libp2p::PeerId::random();
        assert_eq!(
            peer_id_to_player_id(&peer_id),
            peer_id_to_player_id(&peer_id)
        );
    }
}
