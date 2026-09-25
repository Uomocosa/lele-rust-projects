use super::super::constants;
use super::super::directory::room_catalog::RoomCatalog;
use super::super::params::remote_peer_id::RemotePeerId;
use super::hint_store::HintStore;

#[must_use]
pub fn hint_union(
    directory: &RoomCatalog,
    pex: &HintStore,
    own_peer_id: &RemotePeerId,
    now_secs: u64,
) -> Vec<RemotePeerId> {
    let mut peers: Vec<RemotePeerId> = Vec::new();
    for entry in directory.values() {
        push_fresh(
            &mut peers,
            &entry.peer_id,
            *entry.updated_at,
            own_peer_id,
            now_secs,
        );
    }
    for hint in pex.values() {
        push_fresh(
            &mut peers,
            &hint.peer_id,
            *hint.updated_at,
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
    peers: &mut Vec<RemotePeerId>,
    peer_id: &RemotePeerId,
    updated_at: u64,
    own_peer_id: &RemotePeerId,
    now_secs: u64,
) {
    if peer_id.is_empty() || peer_id == own_peer_id {
        return;
    }
    if now_secs.saturating_sub(updated_at) > constants::STALE_ENTRY_SECS {
        return;
    }
    peers.push(peer_id.clone());
}

#[cfg(test)]
mod tests {
    use super::hint_union;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut directory = discovery::directory::RoomCatalog::new();
        directory.insert(
            discovery::params::RoomName("room-a".to_string()),
            discovery::directory::RoomPayload {
                params: discovery::params::ContractParams::default(),
                peer_id: discovery::params::RemotePeerId("peer-a".to_string()),
                addrs: Vec::new(),
                updated_at: discovery::params::EpochSecs(1000),
            },
        );
        directory.insert(
            discovery::params::RoomName("room-b".to_string()),
            discovery::directory::RoomPayload {
                params: discovery::params::ContractParams::default(),
                peer_id: discovery::params::RemotePeerId("own".to_string()),
                addrs: Vec::new(),
                updated_at: discovery::params::EpochSecs(1000),
            },
        );
        let union = hint_union(
            &directory,
            &discovery::gossip::HintStore::default(),
            &discovery::params::RemotePeerId("own".to_string()),
            1000,
        );
        assert_eq!(
            union,
            vec![discovery::params::RemotePeerId("peer-a".to_string())]
        );
    }
}
