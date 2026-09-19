use std::collections::BTreeMap;

use super::mesh_count_count;
use super::mesh_count_is_consistent;
use super::mesh_count_of;
use super::player::Player;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MeshCount {
    pub per: BTreeMap<Player, i32>,
    pub global: i32,
}

#[rustfmt::skip]
impl MeshCount {
    #[must_use]
    pub fn of(pairs: impl IntoIterator<Item = (Player, i32)>) -> Self { mesh_count_of::of(pairs) }
    #[must_use]
    pub fn count(&self, player: impl Into<Player>) -> i32 { mesh_count_count::count(self, player.into()) }
    #[must_use]
    pub fn is_consistent(&self) -> bool { mesh_count_is_consistent::is_consistent(self) }
}

#[cfg(test)]
mod tests {
    use super::MeshCount;
    use super::Player;

    #[test]
    fn test_usage() {
        let count = MeshCount::of([(Player(1), 15), (Player(2), 15), (Player(3), 15)]);
        assert_eq!(count.global, 45);
        assert_eq!(count.count(1), 15);
        assert_eq!(count.count(9), 0);
        assert!(count.is_consistent());
        assert!(MeshCount::default().is_consistent());
    }
}
