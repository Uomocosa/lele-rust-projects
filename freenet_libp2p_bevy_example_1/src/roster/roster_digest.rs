use crate::roster;

/// Renders a roster view as a compact, greppable identity: its length plus the sorted
/// player ids in short hex.
///
/// Counts alone (`entries=2`) cannot distinguish "the same two peers" from "two different
/// peers", so a split-brain is invisible in a single log — it has to be reconstructed by
/// comparing several instances. Two instances printing different digests for the same
/// contract are on disjoint replicas; identical digests mean they genuinely agree.
///
/// The low 32 bits are shown because that is where `p2p::peer_id_to_player_id` concentrates
/// the entropy that actually varies between players.
pub fn roster_digest(entries: &roster::RosterState) -> String {
    let ids: Vec<String> = entries
        .keys()
        .map(|id| format!("{:08x}", **id as u32))
        .collect();
    format!("len={} ids=[{}]", entries.len(), ids.join(","))
}

#[cfg(test)]
mod tests {
    use super::roster_digest;
    use crate::boxes;
    use crate::roster;

    fn entry(peer_id: &str) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            updated_at: 1,
        }
    }

    #[test]
    fn test_usage() {
        let mut entries = roster::RosterState::default();
        entries.insert(boxes::PlayerId(0x11223344), entry("peer-1"));
        entries.insert(boxes::PlayerId(0xaabbccdd), entry("peer-2"));

        assert_eq!(roster_digest(&entries), "len=2 ids=[11223344,aabbccdd]");
    }

    #[test]
    fn digest_is_insertion_order_independent() {
        let mut a = roster::RosterState::default();
        a.insert(boxes::PlayerId(2), entry("b"));
        a.insert(boxes::PlayerId(1), entry("a"));

        let mut b = roster::RosterState::default();
        b.insert(boxes::PlayerId(1), entry("a"));
        b.insert(boxes::PlayerId(2), entry("b"));

        assert_eq!(roster_digest(&a), roster_digest(&b));
    }

    #[test]
    fn same_length_different_peers_differ() {
        let mut a = roster::RosterState::default();
        a.insert(boxes::PlayerId(1), entry("a"));
        a.insert(boxes::PlayerId(2), entry("b"));

        let mut b = roster::RosterState::default();
        b.insert(boxes::PlayerId(1), entry("a"));
        b.insert(boxes::PlayerId(3), entry("c"));

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
