use super::peer_hint_store::HintStore;
use crate::constants;

pub fn prune(store: &mut HintStore, now_secs: u64) {
    store.retain(|_, hint| now_secs.saturating_sub(hint.updated_at) <= constants::STALE_ENTRY_SECS);
}
// no test_usage necessary
