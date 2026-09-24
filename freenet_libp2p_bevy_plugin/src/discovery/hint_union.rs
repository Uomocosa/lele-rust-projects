use super::constants;
use super::directory_state::DirectoryState;
use super::hint_store::HintStore;

#[must_use]
pub fn hint_union(
    directory: &DirectoryState,
    pex: &HintStore,
    own_peer_id: &str,
    now_secs: u64,
) -> Vec<String> {
    let mut peers: Vec<String> = Vec::new();
    for entry in directory.values() {
        push_fresh(
            &mut peers,
            &entry.peer_id,
            entry.updated_at,
            own_peer_id,
            now_secs,
        );
    }
    for hint in pex.values() {
        push_fresh(
            &mut peers,
            &hint.peer_id,
            hint.updated_at,
            own_peer_id,
            now_secs,
        );
    }
    peers.sort();
    peers.dedup();
    peers
}

// needed helper: collects one peer id unless empty, own, or stale
fn push_fresh(
    peers: &mut Vec<String>,
    peer_id: &str,
    updated_at: u64,
    own_peer_id: &str,
    now_secs: u64,
) {
    if peer_id.is_empty() || peer_id == own_peer_id {
        return;
    }
    if now_secs.saturating_sub(updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    peers.push(peer_id.to_string());
}

#[cfg(test)]
mod tests {
    use super::hint_union;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut directory = discovery::DirectoryState::new();
        directory.insert(
            "room-a".to_string(),
            discovery::DirectoryEntry {
                params: Vec::new(),
                peer_id: "peer-a".to_string(),
                addrs: Vec::new(),
                updated_at: 1000,
            },
        );
        directory.insert(
            "room-b".to_string(),
            discovery::DirectoryEntry {
                params: Vec::new(),
                peer_id: "own".to_string(),
                addrs: Vec::new(),
                updated_at: 1000,
            },
        );
        let union = hint_union(&directory, &discovery::HintStore::default(), "own", 1000);
        assert_eq!(union, vec!["peer-a".to_string()]);
    }
}
