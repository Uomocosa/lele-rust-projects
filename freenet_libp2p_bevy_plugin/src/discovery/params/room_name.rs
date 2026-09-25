use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct RoomName(pub String);

#[cfg(test)]
mod tests {
    use super::RoomName;

    #[test]
    fn test_usage() {
        let room = RoomName("room-a".to_string());
        assert_eq!(room.as_str(), "room-a");
        let bytes = bincode::serialize(&room).unwrap_or_default();
        let decoded: RoomName = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, room);
    }
}
