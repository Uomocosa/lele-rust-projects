use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref, Resource,
)]
pub struct NetworkId(pub u64);

#[atomic_delegates]
impl NetworkId {
    pub fn from_peer(peer: &str) -> Self {}
}

#[cfg(test)]
mod tests {
    use super::NetworkId;

    #[test]
    fn test_usage() {
        let id = NetworkId(42);
        assert_eq!(*id, 42);
    }
}
