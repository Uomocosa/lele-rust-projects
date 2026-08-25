use libp2p::identity::Keypair;

use crate::boxes;

pub fn derive_player_id(keypair: &Keypair) -> boxes::PlayerId {
    keypair
        .public()
        .try_into_ed25519()
        .map(|pk| pk.to_bytes())
        .unwrap_or([0u8; 32])
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
