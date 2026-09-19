use clicker_lib::testing;

#[test]
fn single_click_lands_everywhere() {
    let mut mesh = testing::Mesh::of(3);
    mesh.peer(1).click_once();
    for _ in 0..10 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.count(1) == 1),
        "p1 everywhere: {counts:?}"
    );
    assert!(
        counts.iter().all(|c| c.global == 1),
        "global 1 everywhere: {counts:?}"
    );
}
