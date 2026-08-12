use libp2p::identity::Keypair;

use super::peer_id_to_player_id;
use crate::boxes;

pub fn derive_player_id(keypair: &Keypair) -> boxes::PlayerId {
    peer_id_to_player_id(&keypair.public().to_peer_id())
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
