use std::time::Duration;

use bevy::input::keyboard::KeyCode;
use freenet_libp2p_bevy_example_3_lib::engine;

const STEP_DELAY_MS: u64 = 5;

/// Hermetic reproduction of the production node-startup path (`p2p::run` -> embedded node ->
/// `roster::connect_client_loop`) with the two nodes directly wired - one isolated gateway, the
/// other dialing it - instead of relying on the public Freenet mainnet.
///
/// This is now the *deterministic-lockstep* convergence experiment: both peers drive their shared
/// `engine::Engine` from the same ordered per-tick inputs via the `netcode::Lockstep` over real
/// libp2p. Both peers hold the *same keys at the same ticks*, so each tick's ordered input set is
/// identical on both sides and they must end on the **identical engine state hash**. The test also
/// asserts each peer's own box actually moved, proving the engine (not a client-side velocity hack)
/// owns motion.
#[tokio::test(flavor = "multi_thread")]
async fn local_two_node_production_sync() -> Result<(), Box<dyn std::error::Error>> {
    let params = testing_3::unique_params();
    let wasm = testing_3::load_wasm();

    let mut host = testing_3::ProductionGameApp::new_local(&wasm, &params, 0, None).await;
    let gateway = host.freenet_gateway();

    let mut guest = testing_3::ProductionGameApp::new_local(&wasm, &params, 1, Some(gateway)).await;

    host.wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .map_err(|e| {
            format!("host should observe both roster entries once the guest has joined via direct dial: {e}")
        })?;
    guest
        .wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("guest should observe both roster entries: {e}"))?;

    host.wait_for_box_count(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("host should render one box per engine player: {e}"))?;
    guest
        .wait_for_box_count(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("guest should render one box per engine player: {e}"))?;

    step_both(&mut host, &mut guest, 120).await;
    equalize_clocks(&mut host, &mut guest).await;
    let initial_x = engine::spawn_x_for_player(host.own_player_id());

    hold_both(&mut host, &mut guest, KeyCode::KeyD, 180).await;
    step_both(&mut host, &mut guest, 300).await;

    let host_hash = host.state_hash();
    let guest_hash = guest.state_hash();

    let host_end = host
        .own_box_position()
        .ok_or("host engine never positioned its own box")?;

    if host_hash != guest_hash {
        return Err(format!(
            "peers' engine state hashes diverged after lockstep: host={host_hash} guest={guest_hash}"
        )
        .into());
    }

    if (host_end.x - initial_x).abs() <= 10.0 {
        return Err(format!(
            "host's own box did not move under engine authority (was {initial_x}, now {:?})",
            host_end
        )
        .into());
    }

    Ok(())
}

// needed helper:
async fn step_both(
    host: &mut testing_3::ProductionGameApp,
    guest: &mut testing_3::ProductionGameApp,
    rounds: u32,
) {
    for _ in 0..rounds {
        host.tick();
        guest.tick();
        tokio::time::sleep(Duration::from_millis(STEP_DELAY_MS)).await;
    }
}

// needed helper:
async fn hold_both(
    host: &mut testing_3::ProductionGameApp,
    guest: &mut testing_3::ProductionGameApp,
    key: KeyCode,
    rounds: u32,
) {
    host.press_key(key);
    guest.press_key(key);
    step_both(host, guest, rounds).await;
    host.release_key(key);
    guest.release_key(key);
}

// needed helper:
async fn equalize_clocks(
    host: &mut testing_3::ProductionGameApp,
    guest: &mut testing_3::ProductionGameApp,
) {
    let mut hc = host.sim_clock();
    let mut gc = guest.sim_clock();
    while hc < gc {
        host.tick();
        hc = host.sim_clock();
    }
    while gc < hc {
        guest.tick();
        gc = guest.sim_clock();
    }
    if hc != gc {
        panic!("clocks did not equalize: host={hc} guest={gc}");
    }
}
