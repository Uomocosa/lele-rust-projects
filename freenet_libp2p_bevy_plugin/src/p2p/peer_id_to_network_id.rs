use libp2p::PeerId;

use crate::net_id;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Derives the roster key for a peer by hashing its whole `PeerId`.
///
/// FNV-1a is used rather than `DefaultHasher` because this value is part of the shared
/// contract state: every peer must derive the same id for the same `PeerId`, so the hash has
/// to be stable across machines and Rust versions rather than merely stable in-process.
pub fn peer_id_to_network_id(peer_id: &PeerId) -> net_id::NetworkId {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in peer_id.to_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    net_id::NetworkId(hash)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::peer_id_to_network_id;

    #[test]
    fn test_usage() {
        let peer_id = libp2p::PeerId::random();
        assert_eq!(
            peer_id_to_network_id(&peer_id),
            peer_id_to_network_id(&peer_id)
        );
    }

    #[test]
    fn distinct_peers_get_distinct_ids() {
        let ids: HashSet<_> = (0..1000)
            .map(|_| peer_id_to_network_id(&libp2p::PeerId::random()))
            .collect();
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn ids_differ_in_their_high_bits() {
        let high: HashSet<u64> = (0..200)
            .map(|_| *peer_id_to_network_id(&libp2p::PeerId::random()) >> 32)
            .collect();
        assert!(high.len() > 190, "high bits barely varied: {}", high.len());
    }
}
