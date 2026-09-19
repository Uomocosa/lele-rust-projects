use super::mesh_count::MeshCount;
use super::player::Player;

#[must_use]
pub fn of(pairs: impl IntoIterator<Item = (Player, i32)>) -> MeshCount {
    let mut out = MeshCount::default();
    for (player, value) in pairs {
        if value == 0 {
            continue;
        }
        out.global = out.global.saturating_add(value);
        out.per.insert(player, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::MeshCount;
    use super::Player;
    use super::of;

    #[test]
    fn test_usage() {
        let count = of([(Player(1), 1), (Player(2), 5), (Player(3), 17)]);
        assert_eq!(count.global, 23);
        assert_eq!(count.count(2), 5);
        assert_eq!(of([(Player(1), 0)]), MeshCount::default());
    }
}
