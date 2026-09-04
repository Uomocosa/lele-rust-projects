use std::collections::BTreeMap;

use bevy::prelude::Resource;

#[derive(Debug, Clone, Default, Resource)]
pub struct Roster {
    pub lobbies: BTreeMap<String, BTreeMap<[u8; 32], String>>,
}

#[cfg(test)]
mod tests {
    use super::Roster;

    #[test]
    fn test_usage() {
        let r = Roster::default();
        assert!(r.lobbies.is_empty());
    }
}
