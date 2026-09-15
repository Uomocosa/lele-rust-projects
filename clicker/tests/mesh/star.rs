use clicker_lib::testing;

#[test]
fn leaf_to_leaf_click_needs_gossip_relay() {
    let mut mesh = testing::Mesh::three();
    mesh.partition(1, 2);
    mesh.click(2, 5);
    mesh.click(3, 7);
    let want = testing::MeshCount::wanted(0, 5, 7);
    let mut counts = mesh.counts();
    for _ in 0..40 {
        if counts.iter().all(|c| *c == want) {
            break;
        }
        mesh.step();
        counts = mesh.counts();
    }
    assert!(
        counts.iter().all(|c| *c == want),
        "star mesh converged on 0/5/7: {counts:?}"
    );
}

#[test]
fn healed_star_matches_full_mesh() {
    let mut mesh = testing::Mesh::three();
    mesh.partition(1, 2);
    mesh.click(2, 5);
    mesh.click(3, 7);
    mesh.heal(1, 2);
    mesh.click(1, 1);
    let want = testing::MeshCount::wanted(1, 5, 7);
    let mut counts = mesh.counts();
    for _ in 0..40 {
        if counts.iter().all(|c| *c == want) {
            break;
        }
        mesh.step();
        counts = mesh.counts();
    }
    assert!(
        counts.iter().all(|c| *c == want),
        "healed mesh converged on 1/5/7: {counts:?}"
    );
}
