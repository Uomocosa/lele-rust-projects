use crate::discovery;

#[must_use]
pub fn merge_entry(
    existing: Option<discovery::PeerEntry>,
    incoming: discovery::PeerEntry,
) -> discovery::PeerEntry {
    match existing {
        Some(current) if current.updated_at >= incoming.updated_at => current,
        _ => incoming,
    }
}

#[cfg(test)]
mod tests {
    use super::merge_entry;
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
        assert_eq!(merge_entry(None, entry("new", 5)), entry("new", 5));
        assert_eq!(
            merge_entry(Some(entry("old", 9)), entry("new", 5)),
            entry("old", 9)
        );
        assert_eq!(
            merge_entry(Some(entry("old", 5)), entry("new", 9)),
            entry("new", 9)
        );
    }
}
