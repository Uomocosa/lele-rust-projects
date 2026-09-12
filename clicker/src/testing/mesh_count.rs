use super::mesh_count_is_consistent;
use super::mesh_count_wanted;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MeshCount {
    pub p1: i32,
    pub p2: i32,
    pub p3: i32,
    pub global: i32,
}

#[rustfmt::skip]
impl MeshCount {
    #[must_use]
    pub const fn is_consistent(&self) -> bool { mesh_count_is_consistent::is_consistent(self) }
    #[must_use]
    pub const fn wanted(p1: i32, p2: i32, p3: i32) -> Self { mesh_count_wanted::wanted(p1, p2, p3) }
}

#[cfg(test)]
mod tests {
    use super::MeshCount;

    #[test]
    fn test_usage() {
        let count = MeshCount::wanted(15, 15, 15);
        assert_eq!(count.global, 45);
        assert!(count.is_consistent());
        assert!(MeshCount::default().is_consistent());
    }
}
