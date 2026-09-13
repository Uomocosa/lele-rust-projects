use crate::discovery;

#[must_use]
pub fn merge_roster(
    mut base: discovery::RosterState,
    other: discovery::RosterState,
) -> discovery::RosterState {
    for (id, entry) in other {
        let merged = discovery::merge_entry(base.remove(&id), entry);
        base.insert(id, merged);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::merge_roster;
    use crate::discovery;

    fn entry(peer_id: &str, updated_at: u64) -> discovery::PeerEntry {
        discovery::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut base = discovery::RosterState::new();
        base.insert(discovery::PlayerId(1), entry("a", 5));
        let mut other = discovery::RosterState::new();
        other.insert(discovery::PlayerId(1), entry("b", 9));
        other.insert(discovery::PlayerId(2), entry("c", 1));
        let merged = merge_roster(base, other);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged.get(&discovery::PlayerId(1)), Some(&entry("b", 9)));
    }
}
