use std::time::{SystemTime, UNIX_EPOCH};

use freenet_libp2p_bevy_example_3_lib::{boxes, p2p, roster};

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Builds a fresh ed25519 identity with a signed roster entry for a test node.
///
/// The roster contract keys entries by the member's ed25519 public key and requires a valid
/// signature over `(peer_id, addrs, seq)`, so each test node must hold its own keypair
/// (`libp2p::identity::Keypair::generate_ed25519()`), derive its player id from that keypair,
/// and sign its own entry with `seq = now`.
pub fn new_identity(
    peer_id: &str,
) -> (
    libp2p::identity::Keypair,
    boxes::PlayerId,
    roster::PeerEntry,
) {
    let keypair = libp2p::identity::Keypair::generate_ed25519();
    let player_id = p2p::derive_player_id(&keypair);
    let seq = now_unix_secs();
    let entry = roster::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: vec![],
        seq,
        signature: roster::sign_entry(&keypair, peer_id, &[], seq),
    };
    (keypair, player_id, entry)
}

#[cfg(test)]
mod tests {
    use super::new_identity;

    #[test]
    fn test_usage() {
        let (keypair, player_id, entry) = new_identity("peer-1");
        let expected = keypair.public().try_into_ed25519().unwrap().to_bytes();
        assert_eq!(player_id, expected);
        assert_eq!(entry.peer_id, "peer-1");
        assert_eq!(entry.signature.len(), 64);
    }
}
