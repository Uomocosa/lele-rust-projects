use derive_more::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Deref)]
pub struct SyncedPeer(pub String);
