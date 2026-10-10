use libp2p::kad::RecordKey;

use crate::net_id;

#[must_use]
pub fn history_key(room: &net_id::RoomName, chunk: u64) -> RecordKey {
    RecordKey::new(&format!("lobby/history/{}/{:08}", room.as_str(), chunk))
}

#[cfg(test)]
mod tests {
    use super::history_key;
    use crate::net_id;

    #[test]
    fn test_usage() {
        let key = history_key(&net_id::RoomName::from("room-a"), 0);
        assert_ne!(key.as_ref().len(), 0);
    }
}
