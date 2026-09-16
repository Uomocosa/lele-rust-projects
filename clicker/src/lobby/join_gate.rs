use std::collections::BTreeSet;

use bevy::prelude::Resource;

use crate::lobby;

#[derive(Resource, Debug, Default)]
pub struct JoinGate {
    pub expected: Option<BTreeSet<String>>,
    pub synced: Vec<lobby::SyncedPeer>,
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
        gate.expected = Some(BTreeSet::from(["peer-2".to_string()]));
        gate.synced.push(lobby::SyncedPeer("peer-2".to_string()));
        assert!(gate.expected.is_some_and(|set| set.contains("peer-2")));
        assert!(gate.synced.iter().any(|entry| entry.as_str() == "peer-2"));
    }
}
