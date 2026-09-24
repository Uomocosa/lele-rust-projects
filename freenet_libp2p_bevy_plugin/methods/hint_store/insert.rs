use crate::discovery;

pub fn insert(store: &mut discovery::HintStore, hint: discovery::PeerHint) {
    if hint.peer_id.is_empty() || hint.addrs.is_empty() {
        return;
    }
    let keep = store
        .get(&hint.peer_id)
        .is_none_or(|known| hint.updated_at >= known.updated_at);
    if !keep {
        return;
    }
    if !store.contains_key(&hint.peer_id) && store.len() >= discovery::PEX_MAX_HINTS {
        return;
    }
    std::collections::BTreeMap::insert(&mut store.0, hint.peer_id.clone(), hint);
}

#[cfg(test)]
mod tests {
    use super::insert;
    use crate::discovery;

    fn hint(peer_id: &str, updated_at: u64) -> discovery::PeerHint {
        discovery::PeerHint {
            peer_id: peer_id.to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
            rooms: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut store = discovery::HintStore::default();
        insert(&mut store, hint("a", 1));
        assert_eq!(store.len(), 1);
        insert(&mut store, hint("a", 0));
        assert_eq!(store.get("a").map(|h| h.updated_at), Some(1));
        insert(&mut store, hint("", 5));
        assert_eq!(store.len(), 1);
    }
}
