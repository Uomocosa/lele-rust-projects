use clicker_lib::testing;

#[test]
fn agreement_stable_after_converge() {
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
        "setup did not converge: {counts:?}"
    );
    mesh.step();
    mesh.step();
    let later = mesh.counts();
    assert_eq!(counts, later, "agreement drifted");
    assert!(
        later
            .iter()
            .all(clicker_lib::testing::MeshCount::is_consistent),
        "global equals slot sum: {later:?}"
    );
}
