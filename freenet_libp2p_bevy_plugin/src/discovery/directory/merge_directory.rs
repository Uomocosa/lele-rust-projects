use super::merge_directory_entry;
use super::room_catalog::RoomCatalog;

#[must_use]
pub fn merge_directory(mut base: RoomCatalog, other: RoomCatalog) -> RoomCatalog {
    for (room, payload) in other {
        let merged = merge_directory_entry::merge_directory_entry(base.remove(&room), payload);
        base.insert(room, merged);
    }
    base
}

#[cfg(test)]
mod tests {
    use super::merge_directory;
    use crate::discovery;

    fn payload(updated_at: u64) -> discovery::directory::RoomPayload {
        discovery::directory::RoomPayload {
            params: discovery::params::ContractParams(vec![1]),
            peer_id: discovery::params::RemotePeerId("peer".to_string()),
            addrs: Vec::new(),
            updated_at: discovery::params::EpochSecs(updated_at),
        }
    }

    #[test]
    fn test_usage() {
        let mut base = discovery::directory::RoomCatalog::new();
        base.insert(
            discovery::params::RoomName("room-a".to_string()),
            payload(5),
        );
        let mut other = discovery::directory::RoomCatalog::new();
        other.insert(
            discovery::params::RoomName("room-a".to_string()),
            payload(9),
        );
        let merged = merge_directory(base, other);
        assert_eq!(
            merged
                .get(&discovery::params::RoomName("room-a".to_string()))
                .map(|entry| *entry.updated_at),
            Some(9)
        );
    }
}
