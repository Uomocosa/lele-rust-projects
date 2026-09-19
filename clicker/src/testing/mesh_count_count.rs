use super::mesh_count::MeshCount;
use super::player::Player;

#[must_use]
pub fn count(from: &MeshCount, player: Player) -> i32 {
    from.per.get(&player).copied().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::MeshCount;
    use super::Player;
    use super::count;

    #[test]
    fn test_usage() {
        let mesh = MeshCount::of([(Player(1), 4)]);
        assert_eq!(count(&mesh, Player(1)), 4);
        assert_eq!(count(&mesh, Player(2)), 0);
    }
}
