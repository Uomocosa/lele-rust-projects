use crate::roster;

pub fn prune_stale(
    entries: roster::RosterState,
    now_secs: u64,
    ttl_secs: u64,
) -> roster::RosterState {
    entries
        .into_iter()
        .filter(|(_, entry)| now_secs.saturating_sub(entry.seq) <= ttl_secs)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::prune_stale;
    use crate::roster;

    fn entry(seq: u64) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: "peer-1".to_string(),
            addrs: vec![],
            seq,
            signature: Vec::new(),
        }
    }

    #[test]
    fn test_usage() {
        let now = 1_000_000;
        let mut entries = roster::RosterState::default();
        entries.insert([1; 32], entry(now - 10));
        entries.insert([2; 32], entry(now - 1000));

        let pruned = prune_stale(entries, now, 300);

        assert_eq!(pruned.len(), 1);
        assert!(pruned.contains_key(&[1; 32]));
        assert!(!pruned.contains_key(&[2; 32]));
    }
}
