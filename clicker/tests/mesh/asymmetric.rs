use clicker_lib::testing;

#[test]
fn asymmetric_1_5_17_converges() {
    let mut mesh = testing::Mesh::three();
    mesh.click(1, 1);
    mesh.click(2, 5);
    mesh.click(3, 17);
    let want = testing::MeshCount::wanted(1, 5, 17);
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
        "all apps at 1/5/17/23: {counts:?}"
    );
}
