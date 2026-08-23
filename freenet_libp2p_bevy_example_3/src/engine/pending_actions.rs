use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::Deref;

use crate::engine;

#[derive(Resource, Debug, Default, Deref)]
pub struct PendingActions(pub BTreeMap<engine::PlayerId, engine::Action>);

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::PendingActions;
    use crate::engine::Action;

    #[test]
    fn test_usage() {
        let mut map = BTreeMap::new();
        map.insert([1; 32], Action::default());
        let pending = PendingActions(map);
        assert_eq!(pending.len(), 1);
        assert!(pending.contains_key(&[1; 32]));
    }
}
