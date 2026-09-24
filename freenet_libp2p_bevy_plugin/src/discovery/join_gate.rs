use std::collections::BTreeSet;

use bevy::prelude::Resource;

#[derive(Resource, Debug, Default)]
pub struct JoinGate {
    pub expected: Option<BTreeSet<String>>,
    pub committed: bool,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::JoinGate;

    #[test]
    fn test_usage() {
        let mut gate = JoinGate::default();
        assert!(gate.expected.is_none());
        assert!(!gate.committed);
        gate.expected = Some(BTreeSet::from(["peer-2".to_string()]));
        gate.committed = true;
        assert!(gate.expected.is_some_and(|set| set.contains("peer-2")));
        assert!(gate.committed);
    }
}
