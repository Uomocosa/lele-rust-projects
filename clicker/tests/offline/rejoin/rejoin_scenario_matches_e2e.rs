use clicker_lib::testing;
use clicker_lib::testing::scenario::rejoin_scenario;
use clicker_lib::testing::scenario::run_on_mesh;

#[test]
fn rejoin_scenario_matches_e2e() {
    let mut mesh = testing::Mesh::of(3);
    let steps = rejoin_scenario(15);
    assert!(
        run_on_mesh(&mut mesh, &steps).is_ok(),
        "shared rejoin scenario must converge and survive a partition"
    );
}
