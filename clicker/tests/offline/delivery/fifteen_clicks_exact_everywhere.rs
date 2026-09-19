use clicker_lib::testing;

#[test]
fn fifteen_clicks_exact_everywhere() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(2).clicks(15);
    for _ in 0..15 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.count(2) == 15),
        "p2 == 15 everywhere: {counts:?}"
    );
    assert!(
        counts.iter().all(|c| c.global == 15),
        "global == 15 everywhere: {counts:?}"
    );
}
