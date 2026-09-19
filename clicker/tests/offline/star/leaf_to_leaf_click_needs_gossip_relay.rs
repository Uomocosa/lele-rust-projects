use clicker_lib::testing;

#[test]
fn leaf_to_leaf_click_needs_gossip_relay() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(2).partition_from(3);
    mesh.peer(2).clicks(5);
    mesh.peer(3).clicks(7);
    let expected = testing::MeshCount::of([
        (testing::Player(1), 0),
        (testing::Player(2), 5),
        (testing::Player(3), 7),
    ]);
    let counts = mesh.await_convergence(&expected);
    assert!(
        counts.iter().all(|c| *c == expected),
        "star mesh converged on 0/5/7: {counts:?}"
    );
}
