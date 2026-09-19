use clicker_lib::testing;

#[test]
fn full_mesh_converges_45() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(1).clicks(15);
    mesh.peer(2).clicks(15);
    mesh.peer(3).clicks(15);
    let expected = testing::MeshCount::of([
        (testing::Player(1), 15),
        (testing::Player(2), 15),
        (testing::Player(3), 15),
    ]);
    let counts = mesh.await_convergence(&expected);
    assert!(
        counts.iter().all(|c| *c == expected),
        "all apps at 15/15/15/45: {counts:?}"
    );
}
