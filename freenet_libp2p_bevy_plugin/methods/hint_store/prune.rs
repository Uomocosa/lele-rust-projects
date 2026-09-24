use crate::discovery;

pub fn prune(store: &mut discovery::HintStore, now_secs: u64) {
    store.retain(|_, hint| now_secs.saturating_sub(hint.updated_at) <= discovery::STALE_ENTRY_SECS);
}

#[cfg(test)]
mod tests {
    use super::prune;
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
        store.insert(hint("a", 5));
        prune(&mut store, 5 + discovery::STALE_ENTRY_SECS + 1);
        assert!(store.is_empty());
    }
}
