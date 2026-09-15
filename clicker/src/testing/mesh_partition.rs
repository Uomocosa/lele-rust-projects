use super::mesh::Mesh;

pub fn partition(mesh: &mut Mesh, first: usize, second: usize) {
    let pair = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    if !mesh.blocks.contains(&pair) {
        mesh.blocks.push(pair);
    }
}

#[cfg(test)]
mod tests {
    use super::partition;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::three();
        partition(&mut mesh, 1, 2);
        assert!(mesh.blocks.contains(&(1, 2)));
    }
}
