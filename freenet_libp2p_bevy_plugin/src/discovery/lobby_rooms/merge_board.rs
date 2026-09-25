use crate::discovery;
use discovery::lobby_rooms::RoomCatalogue;
use discovery::lobby_rooms::merge_room::merge_room;

#[must_use]
pub fn merge_board(mut base: RoomCatalogue, incoming: RoomCatalogue) -> RoomCatalogue {
    for (room, record) in incoming {
        let merged = merge_room(base.remove(&room), record);
        base.insert(room, merged);
    }
    base
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::merge_board;
    use crate::discovery;

    fn record(peer: &str, updated_at: u64) -> discovery::id::RoomRecord {
        let mut members = BTreeMap::new();
        members.insert(
            discovery::id::RemotePeerId(peer.to_string()),
            discovery::id::Presence {
                addrs: Vec::new(),
                updated_at: discovery::id::EpochSecs(updated_at),
            },
        );
        discovery::id::RoomRecord {
            capacity: 8,
            members,
        }
    }

    #[test]
    fn test_usage() {
        let room = discovery::id::RoomName("room-a".to_string());
        let mut base = BTreeMap::new();
        base.insert(room.clone(), record("peer", 5));
        let mut incoming = BTreeMap::new();
        incoming.insert(room.clone(), record("peer", 9));
        let merged = merge_board(base, incoming);
        let row = merged.get(&room).and_then(|record| {
            record
                .members
                .get(&discovery::id::RemotePeerId("peer".to_string()))
        });
        assert_eq!(row.map(|presence| *presence.updated_at), Some(9));
    }
}
