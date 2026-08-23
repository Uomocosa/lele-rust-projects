use libp2p::identity::Keypair;

use super::peer_id_to_network_id;
use crate::net_id;

pub fn derive_network_id(keypair: &Keypair) -> net_id::NetworkId {
    peer_id_to_network_id(&keypair.public().to_peer_id())
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::derive_network_id;

    #[test]
    fn test_usage() {
        let keypair = Keypair::generate_ed25519();
        let bytes = keypair.to_protobuf_encoding().unwrap();
        let restored = Keypair::from_protobuf_encoding(&bytes).unwrap();
        assert_eq!(derive_network_id(&keypair), derive_network_id(&restored));
    }
}
