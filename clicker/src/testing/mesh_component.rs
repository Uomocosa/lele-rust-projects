use super::mesh::Mesh;

pub fn component(mesh: &Mesh, from: usize) -> Vec<usize> {
    let mut seen = vec![from];
    let mut stack = vec![from];
    while let Some(node) = stack.pop() {
        for peer in 0..mesh.apps.len() {
            if peer != node && !mesh.severed(node, peer) && !seen.contains(&peer) {
                seen.push(peer);
                stack.push(peer);
            }
        }
    }
    seen
}

#[cfg(test)]
mod tests {
    use super::component;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mesh = testing::Mesh::three();
        assert_eq!(component(&mesh, 0).len(), 3);
    }
}
