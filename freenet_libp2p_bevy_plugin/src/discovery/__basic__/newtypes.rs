use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref)]
pub struct GameToken(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deref)]
pub struct GameName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct RoomName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref)]
pub struct RemotePeerId(pub String);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Deref,
)]
pub struct EpochSecs(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct MeshMessage(pub Vec<(RemotePeerId, Vec<String>)>);

#[cfg(test)]
mod tests {
    use super::{EpochSecs, GameName, GameToken, MeshMessage, RemotePeerId, RoomName};

    #[test]
    fn test_usage() {
        assert_eq!(GameToken("token".to_string()).as_str(), "token");
        assert_eq!(GameName("chess".to_string()).as_str(), "chess");
        let room = RoomName("room-a".to_string());
        let bytes = bincode::serialize(&room).unwrap_or_default();
        let decoded: RoomName = bincode::deserialize(&bytes).expect("decodes");
        assert_eq!(decoded, room);
        let peer = RemotePeerId("peer".to_string());
        assert!(!peer.is_empty());
        assert!(EpochSecs(9) > EpochSecs(7));
        let mesh = MeshMessage(vec![(RemotePeerId("p".to_string()), vec![])]);
        assert_eq!(mesh.len(), 1);
    }
}
