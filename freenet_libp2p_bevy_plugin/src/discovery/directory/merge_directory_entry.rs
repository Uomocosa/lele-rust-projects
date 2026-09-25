use super::room_payload::RoomPayload;

#[must_use]
pub fn merge_directory_entry(existing: Option<RoomPayload>, incoming: RoomPayload) -> RoomPayload {
    match existing {
        Some(current) if current.updated_at >= incoming.updated_at => current,
        _ => incoming,
    }
}

#[cfg(test)]
mod tests {
    use super::merge_directory_entry;
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
        assert_eq!(merge_directory_entry(None, payload(5)), payload(5));
        assert_eq!(
            merge_directory_entry(Some(payload(5)), payload(9)),
            payload(9)
        );
    }
}
