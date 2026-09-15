use super::mesh::Mesh;

pub fn severed(mesh: &Mesh, first: usize, second: usize) -> bool {
    let pair = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    mesh.blocks.contains(&pair)
}

#[cfg(test)]
mod tests {
    use super::severed;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mesh = testing::Mesh::three();
        assert!(!severed(&mesh, 1, 2));
    }
}
