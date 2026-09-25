use super::super::directory::DirectoryState;
use super::super::gossip::merge_peer_hints::merge_peer_hints;
use super::super::gossip::peer_hint::PeerHint;

#[must_use]
pub fn directory_hints(slots: &DirectoryState) -> Vec<PeerHint> {
    let hints: Vec<PeerHint> = slots
        .iter()
        .map(|(room, entry)| PeerHint {
            peer_id: entry.peer_id.clone(),
            addrs: entry.addrs.clone(),
            rooms: vec![room.clone()],
            updated_at: entry.updated_at,
        })
        .collect();
    merge_peer_hints(hints)
}

#[cfg(test)]
mod tests {
    use super::directory_hints;

    #[test]
    fn test_usage() {
        use crate::discovery;
        let mut slots = discovery::directory::DirectoryState::new();
        slots.insert(
            "room-a".to_string(),
            discovery::directory::Entry {
                params: vec![1],
                peer_id: "peer-a".to_string(),
                addrs: vec!["/ip4/1.2.3.4/tcp/9000".to_string()],
                updated_at: 100,
            },
        );
        let hints = directory_hints(&slots);
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0].peer_id, "peer-a");
        assert_eq!(hints[0].rooms, vec!["room-a".to_string()]);
    }
}
