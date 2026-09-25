use super::super::directory::room_catalog::RoomCatalog;
use super::super::gossip::merge_peer_hints::merge_peer_hints;
use super::super::gossip::peer_hint::PeerHint;

#[must_use]
pub fn directory_hints(slots: &RoomCatalog) -> Vec<PeerHint> {
    let hints: Vec<PeerHint> = slots
        .iter()
        .map(|(room, payload)| PeerHint {
            peer_id: payload.peer_id.clone(),
            addrs: payload.addrs.clone(),
            rooms: vec![room.clone()],
            updated_at: payload.updated_at,
        })
        .collect();
    merge_peer_hints(hints)
}

#[cfg(test)]
mod tests {
    use super::directory_hints;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut slots = discovery::directory::RoomCatalog::new();
        slots.insert(
            discovery::params::RoomName("room-a".to_string()),
            discovery::directory::RoomPayload {
                params: discovery::params::ContractParams(vec![1]),
                peer_id: discovery::params::RemotePeerId("peer-a".to_string()),
                addrs: vec!["/ip4/1.2.3.4/tcp/9000".to_string()],
                updated_at: discovery::params::EpochSecs(100),
            },
        );
        let hints = directory_hints(&slots);
        assert_eq!(hints.len(), 1);
        assert_eq!(
            hints[0].peer_id,
            discovery::params::RemotePeerId("peer-a".to_string())
        );
        assert_eq!(
            hints[0].rooms,
            vec![discovery::params::RoomName("room-a".to_string())]
        );
    }
}
