use super::mesh::Mesh;
use super::mesh_count::MeshCount;

#[must_use]
pub fn await_convergence(mesh: &mut Mesh, expected: &MeshCount) -> Vec<MeshCount> {
    let mut counts = mesh.counts();
    for _ in 0..Mesh::CONVERGE_TICKS {
        if counts.iter().all(|c| c == expected) {
            break;
        }
        mesh.step();
        counts = mesh.counts();
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::await_convergence;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        mesh.peer(2).clicks(4);
        let expected = testing::MeshCount::of([(testing::Player(2), 4)]);
        let counts = await_convergence(&mut mesh, &expected);
        assert!(counts.iter().all(|c| *c == expected));
    }
}
