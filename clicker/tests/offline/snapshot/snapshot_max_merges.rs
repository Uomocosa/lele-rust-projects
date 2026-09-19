use clicker_lib::testing;

#[test]
fn snapshot_max_merges() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, testing::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    testing::push_to(&mut mesh, 1, testing::history_chunk(vec![(2, 5)], 9));
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.count(2) == 5),
        "snapshot merged to 5: {counts:?}"
    );
}
