use std::collections::BTreeMap;

use atomic_delegate_macros::atomic_delegate;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::peer_hint::PeerHint;

#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct HintStore(pub BTreeMap<String, PeerHint>);

#[atomic_delegate]
impl HintStore {
    pub fn insert(&mut self, hint: PeerHint) {}
    pub fn prune(&mut self, now_secs: u64) {}
}

#[cfg(test)]
mod tests {
    use super::HintStore;
    use crate::discovery;

    fn hint(peer_id: &str, updated_at: u64) -> discovery::gossip::PeerHint {
        discovery::gossip::PeerHint {
            peer_id: peer_id.to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
            rooms: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut store = HintStore::default();
        store.insert(hint("a", 5));
        assert_eq!(store.len(), 1);
        store.insert(hint("a", 3));
        assert_eq!(store.get("a").map(|h| h.updated_at), Some(5));
        store.insert(hint("", 5));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn prune_drops_stale() {
        let mut store = HintStore::default();
        store.insert(hint("a", 5));
        store.prune(5 + discovery::STALE_ENTRY_SECS + 1);
        assert!(store.is_empty());
    }
}
