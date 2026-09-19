use clicker_lib::testing;

#[test]
fn own_snapshot_entry_applies() {
    let mut mesh = testing::Mesh::of(3);
    testing::push_to(&mut mesh, 1, testing::history_chunk(vec![(1, 6)], 6));
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.count(1) == 6),
        "own snapshot entry applied: {counts:?}"
    );
}
