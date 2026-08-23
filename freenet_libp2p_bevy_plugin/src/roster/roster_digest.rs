use crate::roster;

/// Renders a roster view as a compact, greppable identity: its length plus the sorted
/// network ids in short hex.
pub fn roster_digest(entries: &roster::RosterState) -> String {
    let ids: Vec<String> = entries
        .keys()
        .map(|id| format!("{:08x}", **id as u32))
        .collect();
    format!("len={} ids=[{}]", entries.len(), ids.join(","))
}

#[cfg(test)]
mod tests {
    use crate::net_id;

    use super::roster_digest;
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
        entries.insert(net_id::NetworkId(0x11223344), entry("peer-1"));
        entries.insert(net_id::NetworkId(0xaabbccdd), entry("peer-2"));

        assert_eq!(roster_digest(&entries), "len=2 ids=[11223344,aabbccdd]");
    }

    #[test]
    fn digest_is_insertion_order_independent() {
        let mut a = roster::RosterState::default();
        a.insert(net_id::NetworkId(2), entry("b"));
        a.insert(net_id::NetworkId(1), entry("a"));

        let mut b = roster::RosterState::default();
        b.insert(net_id::NetworkId(1), entry("a"));
        b.insert(net_id::NetworkId(2), entry("b"));

        assert_eq!(roster_digest(&a), roster_digest(&b));
    }

    #[test]
    fn empty_roster_has_a_digest() {
        assert_eq!(
            roster_digest(&roster::RosterState::default()),
            "len=0 ids=[]"
        );
    }
}
