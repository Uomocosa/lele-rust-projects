use clicker_lib::testing;

#[test]
fn no_double_credit_on_delivery() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(1).clicks(3);
    for _ in 0..10 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.count(1) == 3 && c.global == 3),
        "exactly 3, no double credit: {counts:?}"
    );
}
