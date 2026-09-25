use super::super::id::presence::Presence;
use super::super::id::room_record::RoomRecord;

#[must_use]
pub fn merge_room(current: Option<RoomRecord>, incoming: RoomRecord) -> RoomRecord {
    match current {
        Some(mut existing) => {
            for (peer, presence) in incoming.members {
                let merged = merge_presence(existing.members.remove(&peer), presence);
                existing.members.insert(peer, merged);
            }
            existing.capacity = existing.capacity.max(incoming.capacity);
            existing
        }
        None => incoming,
    }
}

// needed helper: last-write-wins on the client-supplied timestamp
fn merge_presence(current: Option<Presence>, incoming: Presence) -> Presence {
    match current {
        Some(existing) if existing.updated_at >= incoming.updated_at => existing,
        _ => incoming,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::merge_room;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut members = BTreeMap::new();
        members.insert(
            discovery::id::RemotePeerId("peer".to_string()),
            discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(5),
            },
        );
        let current = discovery::id::RoomRecord {
            capacity: 8,
            members,
        };
        let mut newer = BTreeMap::new();
        newer.insert(
            discovery::id::RemotePeerId("peer".to_string()),
            discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(9),
            },
        );
        let incoming = discovery::id::RoomRecord {
            capacity: 8,
            members: newer,
        };
        let merged = merge_room(Some(current), incoming);
        let row = merged
            .members
            .get(&discovery::id::RemotePeerId("peer".to_string()));
        assert_eq!(row.map(|presence| *presence.updated_at), Some(9));
    }
}
