use clicker_lib::testing;

#[test]
fn asymmetric_1_5_17_converges() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(1).click_once();
    mesh.peer(2).clicks(5);
    mesh.peer(3).clicks(17);
    let expected = testing::MeshCount::of([
        (testing::Player(1), 1),
        (testing::Player(2), 5),
        (testing::Player(3), 17),
    ]);
    let counts = mesh.await_convergence(&expected);
    assert!(
        counts.iter().all(|c| *c == expected),
        "all apps at 1/5/17/23: {counts:?}"
    );
}
