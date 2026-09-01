use crate::roster;

pub fn sign_entry(
    keypair: &libp2p::identity::Keypair,
    peer_id: &str,
    addrs: &[String],
    seq: u64,
) -> Vec<u8> {
    let bytes = roster::entry_signed_bytes(peer_id, addrs, seq);
    keypair.sign(&bytes).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::sign_entry;

    #[test]
    fn test_usage() {
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let sig = sign_entry(&keypair, "peer", &["/b".to_string()], 1);
        assert_eq!(sig.len(), 64);
        assert_ne!(sig, sign_entry(&keypair, "peer", &["/b".to_string()], 2));
    }
}
