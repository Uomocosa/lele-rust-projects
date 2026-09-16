use std::collections::BTreeMap;

use crate::discovery;

#[must_use]
pub fn merge_peer_hints(hints: Vec<discovery::PeerHint>) -> Vec<discovery::PeerHint> {
    let mut newest: BTreeMap<String, discovery::PeerHint> = BTreeMap::new();
    for hint in hints {
        let keep = newest
            .get(&hint.peer_id)
            .is_none_or(|known| hint.updated_at >= known.updated_at);
        if !keep {
            continue;
        }
        let mut merged = hint;
        if let Some(known) = newest.get(&merged.peer_id) {
            for room in &known.rooms {
                if !merged.rooms.contains(room) {
                    merged.rooms.push(room.clone());
                }
            }
        }
        newest.insert(merged.peer_id.clone(), merged);
    }
    newest.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::merge_peer_hints;
    use crate::discovery;

    fn hint(peer_id: &str, updated_at: u64) -> discovery::PeerHint {
        discovery::PeerHint {
            peer_id: peer_id.to_string(),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{updated_at}")],
            rooms: vec!["room-1".to_string()],
            updated_at,
        }
    }

    #[test]
    fn test_usage() {
        let merged = merge_peer_hints(vec![hint("a", 5), hint("a", 9), hint("b", 1)]);
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|h| h.peer_id == "a" && h.updated_at == 9));
        assert!(merged.iter().any(|h| h.peer_id == "b"));
    }

    #[test]
    fn test_rooms_union_on_merge() {
        let mut first = hint("a", 5);
        first.rooms = vec!["room-1".to_string()];
        let mut second = hint("a", 9);
        second.rooms = vec!["room-2".to_string()];
        let merged = merge_peer_hints(vec![first, second]);
        assert_eq!(merged.len(), 1);
        let has_room = |room: &str| {
            merged
                .iter()
                .any(|h| h.peer_id == "a" && h.rooms.iter().any(|r| r == room))
        };
        assert!(has_room("room-1"));
        assert!(has_room("room-2"));
    }
}
