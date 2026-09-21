use crate::constants;
use crate::discovery;

#[must_use]
pub fn hint_union(
    directory: &discovery::DirectoryState,
    pex: &discovery::HintStore,
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

    // needed helper: builds a directory entry for one publisher
    fn dir_entry(peer_id: &str, updated_at: u64) -> discovery::DirectoryEntry {
        discovery::DirectoryEntry {
            params: Vec::new(),
            peer_id: peer_id.to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/4001".to_string()],
            updated_at,
        }
    }

    // needed helper: builds a pex hint for one peer
    fn pex_hint(peer_id: &str, updated_at: u64) -> discovery::PeerHint {
        discovery::PeerHint {
            peer_id: peer_id.to_string(),
            addrs: vec!["/ip4/127.0.0.1/tcp/4002".to_string()],
            rooms: Vec::new(),
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let mut directory = discovery::DirectoryState::new();
        directory.insert("room-a".to_string(), dir_entry("peer-a", 900));
        directory.insert("room-b".to_string(), dir_entry("peer-b", 900));
        let mut pex = discovery::HintStore::default();
        pex.insert(pex_hint("peer-b", 950));
        pex.insert(pex_hint("peer-c", 900));
        let union = hint_union(&directory, &pex, "own", 1000);
        assert_eq!(
            union,
            vec![
                "peer-a".to_string(),
                "peer-b".to_string(),
                "peer-c".to_string()
            ]
        );
    }

    #[test]
    fn drops_own_empty_and_stale() {
        let mut directory = discovery::DirectoryState::new();
        directory.insert("room-a".to_string(), dir_entry("own", 900));
        directory.insert("room-b".to_string(), dir_entry("", 900));
        directory.insert("room-c".to_string(), dir_entry("stale", 100));
        let pex = discovery::HintStore::default();
        let union = hint_union(&directory, &pex, "own", 1000);
        assert!(union.is_empty(), "own/empty/stale ids never join the union");
    }

    #[test]
    fn empty_union_when_nothing_known() {
        let union = hint_union(
            &discovery::DirectoryState::new(),
            &discovery::HintStore::default(),
            "own",
            1000,
        );
        assert!(
            union.is_empty(),
            "fresh creator dials nobody but still resolves"
        );
    }
}
