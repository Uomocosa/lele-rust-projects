use std::collections::BTreeMap;

use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::peer_hint_store_insert;
use super::peer_hint_store_prune;
use crate::discovery;

#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct HintStore(pub BTreeMap<String, discovery::PeerHint>);

#[rustfmt::skip]
impl HintStore {
    pub fn insert(&mut self, hint: discovery::PeerHint) { peer_hint_store_insert::insert(self, hint) }
    pub fn prune(&mut self, now_secs: u64) { peer_hint_store_prune::prune(self, now_secs) }
}

#[cfg(test)]
mod tests {
    use super::HintStore;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut store = HintStore::default();
        store.insert(discovery::PeerHint {
            peer_id: "a".to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/1".to_string()],
            rooms: Vec::new(),
            updated_at: 5,
        });
        assert_eq!(store.len(), 1);
    }
}
