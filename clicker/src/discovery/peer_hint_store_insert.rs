use super::peer_hint_store::HintStore;
use crate::discovery;

pub fn insert(store: &mut HintStore, hint: discovery::PeerHint) {
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
    std::collections::BTreeMap::insert(&mut **store, hint.peer_id.clone(), hint);
}
// no test_usage necessary
