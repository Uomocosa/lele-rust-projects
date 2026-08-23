use std::time::Duration;

use bevy::input::keyboard::KeyCode;

/// Two users starting a session together: app_a boots first, then app_b
/// `JOIN_STAGGER_SECS` later — mirroring real usage, where two humans cannot
/// start an app in the same instant.
const JOIN_STAGGER_SECS: u64 = 45;

/// Reproduces, end-to-end, the manually-observed scenario where instances built through the real
/// production node-startup path (`p2p::load_or_create_keypair` -> `p2p::run` ->
/// `roster::start_embedded_node`, no explicit local gateway wiring) discover each other and sync
/// movement over the public mainnet — unlike the hermetic `TestNode`-based harnesses in
/// `integration_tests`, which point peers directly at a gateway and so never exercise this code
/// path.
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
/// Convergence through the public mainnet DHT remains timing-dependent. The client retries setup
/// with backoff, pulls the roster fast (5 s) during startup, and heartbeats the *full known
/// roster* so a peer that knows more actively heals stale replicas. The deterministic gate for
/// the production startup path is the fully local `local_two_node_production_sync`.
///
/// `#[ignore]`d because it requires public-mainnet access; run explicitly with
/// `cargo test --test e2e_three_node_production_sync -- --ignored`.
#[tokio::test(flavor = "multi_thread")]
#[ignore]
async fn e2e_three_node_production_sync() -> Result<(), Box<dyn std::error::Error>> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "warn,roster=info,p2p=info".into());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();

    testing_3::check_internet_access().await.map_err(|e| {
        format!(
            "this e2e harness requires internet access to reach the public Freenet mainnet: {e}"
        )
    })?;

    let params = testing_3::unique_params();
    let wasm = testing_3::load_wasm();

    let mut app_a = testing_3::ProductionGameApp::new(&wasm, &params, 0).await;
    tokio::time::sleep(Duration::from_secs(JOIN_STAGGER_SECS)).await;
    let mut app_b = testing_3::ProductionGameApp::new(&wasm, &params, 1).await;

    let ids = [app_a.own_player_id(), app_b.own_player_id()];

    for (name, app) in [("app_a", &mut app_a), ("app_b", &mut app_b)] {
        app.wait_for_roster_ids(&ids, Duration::from_secs(120))
            .await
            .map_err(|e| format!("{name} should observe both current-run roster entries: {e}"))?;
        app.wait_for_box_ids(&ids, Duration::from_secs(60))
            .await
            .map_err(|e| {
                format!("{name} should spawn one box per current-run roster entry: {e}")
            })?;
    }

    let initial_a = app_a.box_spawns();

    app_a.simulate_move(KeyCode::KeyD, 30);
    app_b.simulate_move(KeyCode::KeyD, 30);

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
            .ok_or_else(|| format!("box for {id:?} disappeared from app_a"))?;
        if (final_pos.x - initial_pos.x).abs() <= f32::EPSILON {
            return Err(format!(
                "app_a should see remote box {id:?} move after simulate_move + p2p sync \
                 (was {initial_pos:?}, now {final_pos:?})"
            )
            .into());
        }
    }

    Ok(())
}
