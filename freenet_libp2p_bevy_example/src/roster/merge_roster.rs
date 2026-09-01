use crate::roster;

fn merge_entry(
    existing: Option<roster::PeerEntry>,
    incoming: roster::PeerEntry,
) -> roster::PeerEntry {
    match existing {
        Some(current) if current.seq >= incoming.seq => current,
        _ => incoming,
    }
}

pub fn merge_roster(
    mut base: roster::RosterState,
    other: roster::RosterState,
) -> roster::RosterState {
    for (id, entry) in other {
        let merged = merge_entry(base.remove(&id), entry);
        base.insert(id, merged);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::merge_roster;
    use crate::roster;

    fn entry(peer_id: &str, seq: u64) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            seq,
            signature: Vec::new(),
        }
    }

    #[test]
    fn test_usage() {
        let mut base = roster::RosterState::default();
        base.insert([1; 32], entry("peer-1", 5));

        let mut other = roster::RosterState::default();
        other.insert([1; 32], entry("peer-1-stale", 1));
        other.insert([2; 32], entry("peer-2", 9));

        let merged = merge_roster(base, other);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[&[1; 32]].peer_id, "peer-1");
        assert_eq!(merged[&[2; 32]].peer_id, "peer-2");
    }
}
