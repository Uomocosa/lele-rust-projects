use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use super::score_tombstones_keep;
use super::score_tombstones_restore;

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct ScoreTombstones(pub BTreeMap<u64, i32>);

#[rustfmt::skip]
impl ScoreTombstones {
    pub fn keep(&mut self, logical: u64, count: i32) { score_tombstones_keep::keep(self, logical, count) }
    pub fn restore(&mut self, logical: u64) -> Option<i32> { score_tombstones_restore::restore(self, logical) }
}

#[cfg(test)]
mod tests {
    use super::ScoreTombstones;

    #[test]
    fn test_usage() {
        let mut stones = ScoreTombstones::default();
        stones.keep(1, 10);
        stones.keep(1, 4);
        assert_eq!(stones.restore(1), Some(10));
        assert_eq!(stones.restore(1), None);
    }
}
