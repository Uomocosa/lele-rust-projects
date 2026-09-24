use std::collections::BTreeMap;

use super::peer_hint::PeerHint;

#[must_use]
pub fn merge_peer_hints(hints: Vec<PeerHint>) -> Vec<PeerHint> {
    let mut newest: BTreeMap<String, PeerHint> = BTreeMap::new();
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

    #[test]
    fn test_usage() {
        let merged = merge_peer_hints(vec![
            discovery::PeerHint {
                peer_id: "a".to_string(),
                addrs: vec!["/ip4/1/tcp/1".to_string()],
                rooms: vec!["r1".to_string()],
                updated_at: 5,
            },
            discovery::PeerHint {
                peer_id: "a".to_string(),
                addrs: vec!["/ip4/1/tcp/1".to_string()],
                rooms: vec!["r2".to_string()],
                updated_at: 9,
            },
        ]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].rooms.len(), 2);
    }
}
