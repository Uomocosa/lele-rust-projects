use std::collections::BTreeSet;

use bevy::prelude::Resource;
use freenet_libp2p_bevy_plugin::net_id;

use crate::lobby;

#[derive(Resource, Debug, Default)]
pub struct JoinGate {
    pub expected: Option<BTreeSet<String>>,
    pub synced: Vec<lobby::SyncedPeer>,
    pub committed: bool,
    pub pending: Vec<lobby::PendingJoin>,
    pub absent: Vec<String>,
}

#[rustfmt::skip]
impl JoinGate {
    pub fn arm_pending(&mut self, entry: lobby::PendingJoin) { lobby::join_gate_arm_pending::arm_pending(self, entry) }
    #[must_use]
    pub fn has_pending(&self, joiner: net_id::NetworkId) -> bool { lobby::join_gate_has_pending::has_pending(self, joiner) }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::JoinGate;
    use crate::lobby;

    #[test]
    fn test_usage() {
        let mut gate = JoinGate::default();
        assert!(gate.expected.is_none());
        assert_eq!(gate.synced.len(), 0);
        assert!(!gate.committed);
        assert_eq!(gate.pending.len(), 0);
        gate.expected = Some(BTreeSet::from(["peer-2".to_string()]));
        gate.synced.push(lobby::SyncedPeer("peer-2".to_string()));
        gate.committed = true;
        assert!(gate.expected.is_some_and(|set| set.contains("peer-2")));
        assert!(gate.synced.iter().any(|entry| entry.as_str() == "peer-2"));
        assert!(gate.committed);
    }
}
