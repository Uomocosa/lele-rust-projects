use bevy::prelude::Resource;
use derive_more::Deref;
use serde::{Deserialize, Serialize};

use super::network_id_from_peer;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Deref, Resource,
)]
pub struct NetworkId(pub u64);

#[rustfmt::skip]
impl NetworkId {
    #[must_use]
    pub fn from_peer(peer: &str) -> Self { network_id_from_peer::from_peer(peer) }
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
