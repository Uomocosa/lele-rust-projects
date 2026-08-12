use std::time::Duration;

use bevy::input::keyboard::KeyCode;

/// Hermetic reproduction of the production node-startup path (`p2p::load_or_create_keypair` ->
/// `p2p::run` -> `roster::start_embedded_node` -> `roster::connect_client_loop`), but with the two
/// embedded nodes wired directly to each other — one runs as an isolated gateway
/// (`--freenet-local` equivalent) and the other dials it via its `"ip:port,hex-pubkey"` address —
/// instead of relying on the public Freenet mainnet to route the shared roster contract between
/// them.
///
/// This is the decisive experiment for the same-machine sync reliability work: if the two
/// production-path nodes converge when directly wired, the roster contract/merge/subscribe code in
/// this project is correct and residual same-machine flakiness is purely a mainnet
/// node-discovery/bootstrap problem. If this stays red, the bug is in our own code, not upstream.
/// Unlike `e2e_three_node_production_sync.rs`, this test is fully local and needs no internet
/// access.
#[tokio::test(flavor = "multi_thread")]
async fn two_local_production_nodes_converge_and_sync_movement() {
    let params = testing::unique_params();
    let wasm = testing::load_wasm();

    let mut host = testing::ProductionGameApp::new_local(&wasm, &params, 0, None).await;
    let gateway = host.freenet_gateway();

    let mut guest = testing::ProductionGameApp::new_local(&wasm, &params, 1, Some(gateway)).await;

    host.wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .expect(
            "host should observe both roster entries once the guest has joined via direct dial",
        );
    guest
        .wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .expect("guest should observe both roster entries");

    host.wait_for_box_count(2, Duration::from_secs(60))
        .await
        .expect("host should spawn one box per roster entry");
    guest
        .wait_for_box_count(2, Duration::from_secs(60))
        .await
        .expect("guest should spawn one box per roster entry");

    let initial_host = host.box_spawns();

    host.simulate_move(KeyCode::KeyD, 30);
    guest.simulate_move(KeyCode::KeyD, 30);

    for _ in 0..60 {
        host.tick();
        guest.tick();
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let final_host = host.box_spawns();
    for (id, initial_pos, is_local) in &initial_host {
        if *is_local {
            continue;
        }
        let final_pos = final_host
            .iter()
            .find(|(final_id, _, _)| final_id == id)
            .map(|(_, pos, _)| *pos)
            .unwrap_or_else(|| panic!("box for {id:?} disappeared from host"));
        assert!(
            (final_pos.x - initial_pos.x).abs() > f32::EPSILON,
            "host should see guest box {id:?} move after simulate_move + p2p sync \
             (was {initial_pos:?}, now {final_pos:?})"
        );
    }
}
