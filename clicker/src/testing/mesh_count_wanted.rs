use super::mesh_count::MeshCount;

pub const fn wanted(p1: i32, p2: i32, p3: i32) -> MeshCount {
    MeshCount {
        p1,
        p2,
        p3,
        global: p1.saturating_add(p2).saturating_add(p3),
    }
}

#[cfg(test)]
mod tests {
    use super::wanted;

    #[test]
    fn test_usage() {
        let count = wanted(15, 15, 15);
        assert_eq!(count.global, 45);
    }
}
