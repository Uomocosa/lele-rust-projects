use super::mesh_count::MeshCount;

pub const fn is_consistent(count: &MeshCount) -> bool {
    count.global == count.p1.saturating_add(count.p2).saturating_add(count.p3)
}

#[cfg(test)]
mod tests {
    use super::is_consistent;
    use crate::testing;

    #[test]
    fn test_usage() {
        assert!(is_consistent(&testing::MeshCount::wanted(1, 2, 3)));
        assert!(!is_consistent(&testing::MeshCount {
            p1: 1,
            p2: 0,
            p3: 0,
            global: 0,
        }));
    }
}
