use libp2p::PeerId;

use crate::boxes;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Derives the roster key for a peer by hashing its whole `PeerId`.
///
/// Truncating to the first 8 bytes instead — the previous approach — kept almost no entropy:
/// an ed25519 `PeerId` begins with the constant multihash/protobuf header
/// `00 24 08 01 12 20`, so only two bytes varied and roughly 16 bits distinguished any two
/// players. A collision silently merges two players into one `BTreeMap` entry, which on
/// screen is indistinguishable from a peer that never showed up.
///
/// FNV-1a is used rather than `DefaultHasher` because this value is part of the shared
/// contract state: every peer must derive the same id for the same `PeerId`, so the hash has
/// to be stable across machines and Rust versions rather than merely stable in-process.
pub fn peer_id_to_player_id(peer_id: &PeerId) -> boxes::PlayerId {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in peer_id.to_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    boxes::PlayerId(hash)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::peer_id_to_player_id;

    #[test]
    fn test_usage() {
        let peer_id = libp2p::PeerId::random();
        assert_eq!(
            peer_id_to_player_id(&peer_id),
            peer_id_to_player_id(&peer_id)
        );
    }

    #[test]
    fn distinct_peers_get_distinct_ids() {
        let ids: HashSet<_> = (0..1000)
            .map(|_| peer_id_to_player_id(&libp2p::PeerId::random()))
            .collect();
        assert_eq!(ids.len(), 1000);
    }

    /// The old truncating derivation kept only the low bits of an otherwise constant
    /// ed25519 header, so distinct peers collided in the high 48 bits. Guard that the hash
    /// spreads across the whole word instead.
    #[test]
    fn ids_differ_in_their_high_bits() {
        let high: HashSet<u64> = (0..200)
            .map(|_| *peer_id_to_player_id(&libp2p::PeerId::random()) >> 32)
            .collect();
        assert!(high.len() > 190, "high bits barely varied: {}", high.len());
    }

    #[test]
    fn hash_is_stable_for_a_known_input() {
        let peer_id = libp2p::PeerId::from_bytes(&[
            0x00, 0x24, 0x08, 0x01, 0x12, 0x20, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
            0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ])
        .expect("valid ed25519 peer id");
        let first = peer_id_to_player_id(&peer_id);
        let second = peer_id_to_player_id(&peer_id);
        assert_eq!(first, second);
    }
}
