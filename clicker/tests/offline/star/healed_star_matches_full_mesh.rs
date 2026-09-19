use clicker_lib::testing;

#[test]
fn healed_star_matches_full_mesh() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(2).partition_from(3);
    mesh.peer(2).clicks(5);
    mesh.peer(3).clicks(7);
    mesh.peer(2).heal_with(3);
    mesh.peer(1).click_once();
    let expected = testing::MeshCount::of([
        (testing::Player(1), 1),
        (testing::Player(2), 5),
        (testing::Player(3), 7),
    ]);
    let counts = mesh.await_convergence(&expected);
    assert!(
        counts.iter().all(|c| *c == expected),
        "healed mesh converged on 1/5/7: {counts:?}"
    );
}
