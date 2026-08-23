use crate::roster;

fn merge_entry(
    existing: Option<roster::PeerEntry>,
    incoming: roster::PeerEntry,
) -> roster::PeerEntry {
    match existing {
        Some(current) if current.updated_at >= incoming.updated_at => current,
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
    use crate::net_id;

    use super::merge_roster;
    use crate::roster;

    fn entry(peer_id: &str, updated_at: u64) -> roster::PeerEntry {
        roster::PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![],
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut base = roster::RosterState::default();
        base.insert(net_id::NetworkId(1), entry("peer-1", 5));

        let mut other = roster::RosterState::default();
        other.insert(net_id::NetworkId(1), entry("peer-1-stale", 1));
        other.insert(net_id::NetworkId(2), entry("peer-2", 9));

        let merged = merge_roster(base, other);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[&net_id::NetworkId(1)].peer_id, "peer-1");
        assert_eq!(merged[&net_id::NetworkId(2)].peer_id, "peer-2");
    }
}
