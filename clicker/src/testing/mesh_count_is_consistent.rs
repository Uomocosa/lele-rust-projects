use super::mesh_count::MeshCount;

#[must_use]
pub fn is_consistent(count: &MeshCount) -> bool {
    let sum = count.per.values().copied().fold(0, i32::saturating_add);
    count.global == sum
}

#[cfg(test)]
mod tests {
    use super::super::player::Player;
    use super::{MeshCount, is_consistent};

    #[test]
    fn test_usage() {
        assert!(is_consistent(&MeshCount::of([
            (Player(1), 1),
            (Player(2), 2),
            (Player(3), 3),
        ])));
        assert!(!is_consistent(&MeshCount {
            per: std::iter::once((Player(1), 1)).collect(),
            global: 0,
        }));
    }
}
