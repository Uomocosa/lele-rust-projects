use std::time::Duration;

use bevy::input::keyboard::KeyCode;

/// Two users starting a session together: app_a boots first, then app_b
/// `JOIN_STAGGER_SECS` later — mirroring real usage, where two humans cannot
/// start an app in the same instant.
const JOIN_STAGGER_SECS: u64 = 45;

/// Reproduces, end-to-end, the manually-observed scenario where instances built through the real
/// production node-startup path (`p2p::load_or_create_keypair` -> `p2p::run` ->
/// `roster::start_embedded_node`, no explicit local gateway wiring) discover each other and sync
/// movement over the public mainnet — unlike the hermetic `TestNode`-based tests in
/// `two_node_roster.rs` / `two_node_box_count.rs`, which point peers directly at a gateway and so
/// never exercise this code path.
///
/// Uses a fresh contract key per run plus a **staggered join** (app_b starts
/// `JOIN_STAGGER_SECS` after app_a), which is what the production
/// `setup_contract` grace window (`roster::SETUP_CONTRACT_GRACE_SECS`) is designed for: app_a's
/// `Put` seeds the key after its grace window expires, and app_b — still inside its own window —
/// finds that seed and merges instead of racing a second `Put`. Two *concurrent* first `Put`s of
/// the same fresh key can seed disjoint replicas that only reconcile via freenet's 5-minute
/// InterestSync anti-entropy (`INTEREST_HEARTBEAT_INTERVAL`), which is upstream behaviour, not
/// this project's code — see OBJECTIVE.md.
///
/// Ignored by default: convergence through the public mainnet DHT remains timing-dependent.
/// The client retries setup with backoff, pulls the roster fast (5 s) during startup, and
/// heartbeats the *full known roster* so a peer that knows more actively heals stale replicas.
/// The deterministic gate for the production startup path is the fully local
/// `local_two_node_production_sync.rs`, which wires the same code directly. Run this test
/// explicitly with `-- --ignored` to probe the live mainnet.
///
/// Waits and movement assertions scope to the *current run's* two player ids, not to roster/box
/// counts, so stale entries accumulated on the shared contract key from earlier runs cannot
/// satisfy them.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn two_production_nodes_converge_and_sync_movement() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,roster=info,p2p=info".into());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    testing::check_internet_access()
        .await
        .expect("this e2e test requires internet access to reach the public Freenet mainnet");

    let params = testing::unique_params();
    let wasm = testing::load_wasm();

    let mut app_a = testing::ProductionGameApp::new(&wasm, &params, 0).await;
    tokio::time::sleep(Duration::from_secs(JOIN_STAGGER_SECS)).await;
    let mut app_b = testing::ProductionGameApp::new(&wasm, &params, 1).await;

    let ids = [app_a.own_player_id(), app_b.own_player_id()];

    // Step 1: every app must observe both current-run players in the roster and spawn a box for
    // each. Scoped by id, so stale entries from previous runs can't satisfy the wait.
    for (name, app) in [("app_a", &mut app_a), ("app_b", &mut app_b)] {
        app.wait_for_roster_ids(&ids, Duration::from_secs(120))
            .await
            .unwrap_or_else(|_| panic!("{name} should observe both current-run roster entries"));
        app.wait_for_box_ids(&ids, Duration::from_secs(60))
            .await
            .unwrap_or_else(|_| panic!("{name} should spawn one box per current-run roster entry"));
    }

    // Step 2: snapshot spawn positions before moving anything (spawn x is spread out by
    // `boxes::pick_spawn_x`, not a single fixed constant, so "moved" must be judged relative to
    // each box's own recorded starting position, not an absolute value).
    let initial_a = app_a.box_spawns();

    // Step 3: each app moves its own local box.
    app_a.simulate_move(KeyCode::KeyD, 30);
    app_b.simulate_move(KeyCode::KeyD, 30);

    // Step 4: after direct p2p snapshot sync, every app should see the other box have moved off
    // its recorded spawn position.
    for _ in 0..60 {
        app_a.tick();
        app_b.tick();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let final_a = app_a.box_spawns();
    for (id, initial_pos, is_local) in &initial_a {
        if *is_local || !ids.contains(id) {
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
