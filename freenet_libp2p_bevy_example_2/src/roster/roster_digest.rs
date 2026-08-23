use crate::roster;

/// Renders a roster view as a compact, greppable identity: its length plus the sorted
/// player ids as short hex.
///
/// Counts alone (`entries=2`) cannot distinguish "the same two peers" from "two different
/// peers", so a split-brain is invisible in a single log — it has to be reconstructed by
/// comparing several instances. Two instances printing different digests for the same
/// contract are on disjoint replicas; identical digests mean they genuinely agree.
///
/// The first 8 bytes of the 32-byte ed25519 pubkey are shown; the pubkey is uniformly random
/// so those suffice to tell ids apart.
pub fn roster_digest(entries: &roster::RosterState) -> String {
    let ids: Vec<String> = entries
        .keys()
        .map(|id| id[0..8].iter().map(|b| format!("{b:02x}")).collect())
        .collect();
    format!("len={} ids=[{}]", entries.len(), ids.join(","))
}

#[cfg(test)]
mod tests {
    use super::roster_digest;
    use crate::roster;

    fn entry(peer_id: &str) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            seq: 1,
            signature: Vec::new(),
        }
    }

    fn key(prefix: [u8; 8]) -> [u8; 32] {
        let mut k = [0u8; 32];
        k[..8].copy_from_slice(&prefix);
        k
    }

    #[test]
    fn test_usage() {
        let mut entries = roster::RosterState::default();
        entries.insert(
            key([0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]),
            entry("peer-1"),
        );
        entries.insert(
            key([0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11]),
            entry("peer-2"),
        );

        assert_eq!(
            roster_digest(&entries),
            "len=2 ids=[1122334455667788,aabbccddeeff0011]"
        );
    }

    #[test]
    fn digest_is_insertion_order_independent() {
        let mut a = roster::RosterState::default();
        a.insert(key([2, 0, 0, 0, 0, 0, 0, 0]), entry("b"));
        a.insert(key([1, 0, 0, 0, 0, 0, 0, 0]), entry("a"));

        let mut b = roster::RosterState::default();
        b.insert(key([1, 0, 0, 0, 0, 0, 0, 0]), entry("a"));
        b.insert(key([2, 0, 0, 0, 0, 0, 0, 0]), entry("b"));

        assert_eq!(roster_digest(&a), roster_digest(&b));
    }

    #[test]
    fn same_length_different_peers_differ() {
        let mut a = roster::RosterState::default();
        a.insert(key([1, 0, 0, 0, 0, 0, 0, 0]), entry("a"));
        a.insert(key([2, 0, 0, 0, 0, 0, 0, 0]), entry("b"));

        let mut b = roster::RosterState::default();
        b.insert(key([1, 0, 0, 0, 0, 0, 0, 0]), entry("a"));
        b.insert(key([3, 0, 0, 0, 0, 0, 0, 0]), entry("c"));

        assert_ne!(roster_digest(&a), roster_digest(&b));
    }

    #[test]
    fn empty_roster_has_a_digest() {
        assert_eq!(
            roster_digest(&roster::RosterState::default()),
            "len=0 ids=[]"
        );
    }
}
