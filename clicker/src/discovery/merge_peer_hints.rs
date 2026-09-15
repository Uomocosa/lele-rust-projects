use std::collections::BTreeMap;

use crate::discovery;

#[must_use]
pub fn merge_peer_hints(hints: Vec<discovery::PeerHint>) -> Vec<discovery::PeerHint> {
    let mut newest: BTreeMap<String, discovery::PeerHint> = BTreeMap::new();
    for hint in hints {
        let keep = newest
            .get(&hint.peer_id)
            .is_none_or(|known| hint.updated_at > known.updated_at);
        if keep {
            newest.insert(hint.peer_id.clone(), hint);
        }
    }
    newest.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::merge_peer_hints;
    use crate::discovery;

    fn hint(peer_id: &str, updated_at: u64) -> discovery::PeerHint {
        discovery::PeerHint {
            peer_id: peer_id.to_string(),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{updated_at}")],
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let merged = merge_peer_hints(vec![hint("a", 5), hint("a", 9), hint("b", 1)]);
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|h| h.peer_id == "a" && h.updated_at == 9));
        assert!(merged.iter().any(|h| h.peer_id == "b"));
    }
}
