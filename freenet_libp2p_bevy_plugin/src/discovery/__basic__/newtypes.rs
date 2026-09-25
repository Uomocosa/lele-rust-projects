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
