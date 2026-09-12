use clicker_lib::testing;

fn converged(counts: &[testing::MeshCount]) -> bool {
    let want = testing::MeshCount {
        p1: 15,
        p2: 15,
        p3: 15,
        global: 45,
    };
    counts.iter().all(|c| *c == want)
}

#[test]
fn full_mesh_converges_45() {
    let mut mesh = testing::Mesh::three();
    mesh.click(1, 15);
    mesh.click(2, 15);
    mesh.click(3, 15);
    let mut counts = mesh.counts();
    for _ in 0..20 {
        if converged(&counts) {
            break;
        }
        mesh.step();
        counts = mesh.counts();
    }
    assert!(converged(&counts), "all apps at 15/15/15/45: {counts:?}");
}

#[test]
fn agreement_stable_after_converge() {
    let mut mesh = testing::Mesh::three();
    mesh.click(1, 15);
    mesh.click(2, 15);
    mesh.click(3, 15);
    let mut counts = mesh.counts();
    for _ in 0..20 {
        if converged(&counts) {
            break;
        }
        mesh.step();
        counts = mesh.counts();
    }
    assert!(converged(&counts), "setup did not converge: {counts:?}");
    mesh.step();
    mesh.step();
    let later = mesh.counts();
    assert_eq!(counts, later, "agreement drifted");
    assert!(
        later.iter().all(|c| c.global == c.p1 + c.p2 + c.p3),
        "global equals slot sum: {later:?}"
    );
}
