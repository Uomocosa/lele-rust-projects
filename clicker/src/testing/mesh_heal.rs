use super::mesh::Mesh;

pub fn heal(mesh: &mut Mesh, first: usize, second: usize) {
    let pair = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    mesh.blocks.retain(|blocked| *blocked != pair);
}

#[cfg(test)]
mod tests {
    use super::heal;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::three();
        mesh.partition(0, 1);
        heal(&mut mesh, 0, 1);
        assert!(!mesh.blocks.contains(&(0, 1)));
    }
}
