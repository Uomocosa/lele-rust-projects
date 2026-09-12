use clicker_lib::testing;

#[test]
fn single_click_lands_everywhere() {
    let mut mesh = testing::Mesh::three();
    mesh.click(1, 1);
    for _ in 0..10 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.p1 == 1),
        "p1 everywhere: {counts:?}"
    );
    assert!(
        counts.iter().all(|c| c.global == 1),
        "global 1 everywhere: {counts:?}"
    );
}

#[test]
fn fifteen_clicks_exact_everywhere() {
    let mut mesh = testing::Mesh::three();
    mesh.click(2, 15);
    for _ in 0..15 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.p2 == 15),
        "p2 == 15 everywhere: {counts:?}"
    );
    assert!(
        counts.iter().all(|c| c.global == 15),
        "global == 15 everywhere: {counts:?}"
    );
}

#[test]
fn no_double_credit_on_delivery() {
    let mut mesh = testing::Mesh::three();
    mesh.click(1, 3);
    for _ in 0..10 {
        mesh.step();
    }
    let counts = mesh.counts();
    assert!(
        counts.iter().all(|c| c.p1 == 3 && c.global == 3),
        "exactly 3, no double credit: {counts:?}"
    );
}
