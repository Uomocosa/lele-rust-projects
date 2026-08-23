use libp2p::PeerId;

use crate::boxes;

/// Returns the raw ed25519 public key bytes of an ed25519 `PeerId`.
///
/// The ed25519 `PeerId`'s `to_bytes()` is the multihash of the protobuf-encoded public key:
/// the fixed 6-byte header `00 24 08 01 12 20` followed by the 32-byte ed25519 public key.
/// Slicing off the header yields the exact `[u8; 32]` roster key the contract uses.
pub fn peer_id_to_player_id(peer_id: &PeerId) -> boxes::PlayerId {
    let bytes = peer_id.to_bytes();
    let mut id = [0u8; 32];
    if bytes.len() >= 38 {
        id.copy_from_slice(&bytes[6..38]);
    }
    id
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::peer_id_to_player_id;

    #[test]
    fn test_usage() {
        let keypair = Keypair::generate_ed25519();
        let peer_id = keypair.public().to_peer_id();
        let expected = keypair.public().try_into_ed25519().unwrap().to_bytes();
        assert_eq!(peer_id_to_player_id(&peer_id), expected);
    }
}
