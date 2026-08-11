use libp2p::identity::Keypair;

use crate::boxes;

pub fn derive_player_id(keypair: &Keypair) -> boxes::PlayerId {
    let bytes = keypair.public().to_peer_id().to_bytes();
    let mut id_bytes = [0u8; 8];
    id_bytes.copy_from_slice(&bytes[..8]);
    boxes::PlayerId(u64::from_be_bytes(id_bytes))
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::derive_player_id;

    #[test]
    fn test_usage() {
        let keypair = Keypair::generate_ed25519();
        let bytes = keypair.to_protobuf_encoding().unwrap();
        let restored = Keypair::from_protobuf_encoding(&bytes).unwrap();
        assert_eq!(derive_player_id(&keypair), derive_player_id(&restored));
    }
}
