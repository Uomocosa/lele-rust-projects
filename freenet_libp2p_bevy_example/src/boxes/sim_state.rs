use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;

use crate::engine;
use crate::p2p;

#[derive(Resource, Default)]
pub struct SimState {
    pub clock: u64,
    pub latest_hash: Option<u64>,
    pub peer_hashes: BTreeMap<engine::PlayerId, BTreeMap<u64, u64>>,
    pub divergence_count: u64,
    pub last_adopted_tick: u64,
    pub seen_peers: BTreeSet<engine::PlayerId>,
    pub pending_reveals: BTreeMap<u64, engine::Action>,
    pub pending_snapshots: Vec<p2p::NetcodeMsg>,
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
        assert_eq!(state.divergence_count, 0);
        assert_eq!(state.last_adopted_tick, 0);
        assert!(state.seen_peers.is_empty());
        assert!(state.pending_reveals.is_empty());
    }
}
