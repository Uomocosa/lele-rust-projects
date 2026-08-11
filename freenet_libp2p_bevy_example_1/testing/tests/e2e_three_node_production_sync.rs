use std::time::Duration;

use bevy::input::keyboard::KeyCode;

/// Reproduces, end-to-end, the manually-observed bug where instances built through the real
/// production node-startup path (`p2p::load_or_create_keypair` -> `p2p::run` ->
/// `roster::start_embedded_node`, no explicit local gateway wiring) fail to reliably discover
/// each other, unlike the hermetic `TestNode`-based tests in `two_node_roster.rs` /
/// `two_node_box_count.rs`, which point peers directly at a gateway and so never exercise this
/// code path.
///
/// This test is expected to be RED today: step 1 (roster convergence across 3 production-path
/// nodes) times out. Steps 2-3 (movement sync) encode the target end state once the underlying
/// node-discovery bug is fixed and are not expected to run to a meaningful assertion yet.
#[tokio::test(flavor = "multi_thread")]
async fn three_production_nodes_converge_and_sync_movement() {
    testing::check_internet_access()
        .await
        .expect("this e2e test requires internet access to reach the public Freenet mainnet");

    let params = testing::unique_params();
    let wasm = testing::load_wasm();

    let mut app_a = testing::ProductionGameApp::new(&wasm, &params, 0).await;
    let mut app_b = testing::ProductionGameApp::new(&wasm, &params, 1).await;
    let mut app_c = testing::ProductionGameApp::new(&wasm, &params, 2).await;

    // Step 1: roster convergence. Known-red today — see module doc comment. Production
    // instances join the public mainnet independently, with no direct link to each other,
    // so they may never observe each other's roster entry within any bounded timeout.
    app_a
        .wait_for_roster_len(3, Duration::from_secs(60))
        .await
        .expect(
            "app_a should observe all 3 roster entries — if this fails, that's the \
             production node-discovery bug (see module doc comment), not a flake",
        );
    app_b
        .wait_for_roster_len(3, Duration::from_secs(60))
        .await
        .expect("app_b should observe all 3 roster entries");
    app_c
        .wait_for_roster_len(3, Duration::from_secs(60))
        .await
        .expect("app_c should observe all 3 roster entries");

    app_a
        .wait_for_box_count(3, Duration::from_secs(60))
        .await
        .expect("app_a should spawn one box per roster entry");
    app_b
        .wait_for_box_count(3, Duration::from_secs(60))
        .await
        .expect("app_b should spawn one box per roster entry");
    app_c
        .wait_for_box_count(3, Duration::from_secs(60))
        .await
        .expect("app_c should spawn one box per roster entry");

    // Step 2: snapshot spawn positions before moving anything (spawn x is spread out by
    // `boxes::pick_spawn_x`, not a single fixed constant, so "moved" must be judged relative to
    // each box's own recorded starting position, not an absolute value).
    let initial_a = app_a.box_spawns();

    // Step 3: each app moves its own local box.
    app_a.simulate_move(KeyCode::KeyD, 30);
    app_b.simulate_move(KeyCode::KeyD, 30);
    app_c.simulate_move(KeyCode::KeyD, 30);

    // Step 4: after direct p2p snapshot sync, every app should see the other two boxes have
    // moved off their recorded spawn position.
    for _ in 0..60 {
        app_a.tick();
        app_b.tick();
        app_c.tick();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let final_a = app_a.box_spawns();
    for (id, initial_pos, is_local) in &initial_a {
        if *is_local {
            continue;
        }
        let final_pos = final_a
            .iter()
            .find(|(final_id, _, _)| final_id == id)
            .map(|(_, pos, _)| *pos)
            .unwrap_or_else(|| panic!("box for {id:?} disappeared from app_a"));
        assert!(
            (final_pos.x - initial_pos.x).abs() > f32::EPSILON,
            "app_a should see remote box {id:?} move after simulate_move + p2p sync \
             (was {initial_pos:?}, now {final_pos:?})"
        );
    }
}
