use clicker_lib::testing;

use crate::support;

#[test]
fn snapshot_max_merges() {
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    support::push_to(&mut mesh, 1, support::snapshot_chunk(vec![(2, 5)], 9));
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.p2 == 5),
        "snapshot merged to 5: {counts:?}"
    );
}

#[test]
fn stale_snapshot_ignored() {
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, support::move_gossip("peer-2", 2));
    for _ in 0..3 {
        mesh.step();
    }
    support::push_to(&mut mesh, 1, support::snapshot_chunk(vec![(2, 5)], 9));
    mesh.step();
    support::push_to(&mut mesh, 1, support::snapshot_chunk(vec![(2, 3)], 9));
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().any(|c| c.p2 == 5),
        "stale snapshot kept at 5: {counts:?}"
    );
}

#[test]
fn unlabeled_snapshot_drops() {
    let mut mesh = testing::Mesh::three();
    support::push_to(&mut mesh, 1, support::snapshot_chunk(vec![(2, 5)], 9));
    mesh.step();
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| *c == testing::MeshCount::default()),
        "unlabeled snapshot dropped: {counts:?}"
    );
}
