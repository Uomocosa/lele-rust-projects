use clicker_lib::testing;

#[test]
fn unlabeled_snapshot_drops() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, testing::history_chunk(vec![(2, 5)], 9));
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| *c == testing::MeshCount::default()),
        "unlabeled snapshot dropped: {counts:?}"
    );
}
