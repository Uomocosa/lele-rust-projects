use std::collections::BTreeSet;

use bevy::prelude::Resource;

use super::super::params::remote_peer_id::RemotePeerId;

#[derive(Resource, Debug, Default)]
pub struct JoinGate {
    pub expected: Option<BTreeSet<RemotePeerId>>,
    pub committed: bool,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::JoinGate;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut gate = JoinGate::default();
        assert!(gate.expected.is_none());
        assert!(!gate.committed);
        gate.expected = Some(BTreeSet::from([discovery::params::RemotePeerId(
            "peer-2".to_string(),
        )]));
        gate.committed = true;
        assert!(gate.expected.is_some_and(|set| {
            set.contains(&discovery::params::RemotePeerId("peer-2".to_string()))
        }));
        assert!(gate.committed);
    }
}
