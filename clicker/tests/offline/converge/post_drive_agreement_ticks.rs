use clicker_lib::testing;

#[test]
fn post_drive_agreement_ticks() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(1).clicks(15);
    mesh.peer(2).clicks(15);
    mesh.peer(3).clicks(15);
    let expected = testing::MeshCount::of([
        (testing::Player(1), 15),
        (testing::Player(2), 15),
        (testing::Player(3), 15),
    ]);
    let mut ticks = 0;
    let mut counts = mesh.counts();
    while ticks < testing::Mesh::CONVERGE_TICKS {
        if counts.iter().all(|c| *c == expected) {
            break;
        }
        mesh.step();
        counts = mesh.counts();
        ticks += 1;
    }
    assert!(
        counts.iter().all(|c| *c == expected),
        "post-drive 15/15/15 must agree, took {ticks} ticks: {counts:?}"
    );
    assert!(
        ticks < testing::Mesh::CONVERGE_TICKS,
        "post-drive agreement must land well inside the converge window, took {ticks}"
    );
}
