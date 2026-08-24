use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;

use crate::engine;

#[derive(Resource, Default)]
pub struct SimState {
    pub clock: u64,
    pub latest_hash: Option<u64>,
    pub peer_hashes: BTreeMap<engine::PlayerId, BTreeMap<u64, u64>>,
    pub seen_peers: BTreeSet<engine::PlayerId>,
}

#[cfg(test)]
mod tests {
    use super::SimState;

    #[test]
    fn test_usage() {
        let state = SimState::default();
        assert_eq!(state.clock, 0);
        assert!(state.latest_hash.is_none());
        assert!(state.peer_hashes.is_empty());
        assert!(state.seen_peers.is_empty());
    }
}
