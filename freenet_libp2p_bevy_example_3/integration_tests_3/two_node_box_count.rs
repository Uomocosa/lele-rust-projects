use std::time::Duration;

use freenet_libp2p_bevy_example_3_lib::engine;
use libp2p::identity::Keypair;

// needed helper:
fn assert_boxes(app: &mut testing_3::TestGameApp) -> Result<(), String> {
    assert_eq!(app.box_count(), 2);
    assert_eq!(app.roster_len(), 2);

    let spawns = app.box_spawns();
    assert_eq!(spawns.len(), 2);

    let locals: Vec<_> = spawns.iter().filter(|(_, _, is_local)| *is_local).collect();
    let remotes: Vec<_> = spawns
        .iter()
        .filter(|(_, _, is_local)| !*is_local)
        .collect();
    assert_eq!(locals.len(), 1, "exactly one local player box");
    assert_eq!(remotes.len(), 1, "exactly one remote box");

    let remote = remotes[0];
    let resting = engine::GROUND_TOP + engine::BOX_SIZE / 2.0;
    assert!(
        (remote.1.y - resting).abs() < 10.0,
        "remote box must rest on the ground (was {})",
        remote.1.y
    );

    let mut xs: Vec<f32> = spawns.iter().map(|(_, pos, _)| pos.x).collect();
    xs.sort_by(f32::total_cmp);
    assert!(xs[0] != xs[1], "box x positions must be distinct");
    let bound = engine::GROUND_WIDTH / 2.0 - engine::BOX_SIZE / 2.0;
    assert!(
        xs[0] >= -bound && xs[1] <= bound,
        "box x positions must stay on the ground"
    );
    Ok(())
}

/// Two instances of the game run on two embedded nodes that join the same *private* roster
/// contract (unique params). Both converge on a 2-entry roster, and each app ends up with exactly
/// 2 boxes: 1 local player plus 1 kinematic remote box, spread out over the ground.
#[tokio::test(flavor = "multi_thread")]
async fn two_node_box_count() -> Result<(), Box<dyn std::error::Error>> {
    let params = testing_3::unique_params();
    let gateway = testing_3::TestNode::start_gateway(0)
        .await
        .map_err(|e| format!("gateway node should start: {e}"))?;
    let peer = testing_3::TestNode::start_peer(gateway.public_port(), gateway.public_key_hex())
        .await
        .map_err(|e| format!("peer node should join the gateway: {e}"))?;
    let wasm = testing_3::load_wasm();

    let mut gateway_app = testing_3::TestGameApp::new(
        gateway.port(),
        &wasm,
        &params,
        Keypair::generate_ed25519(),
        "gateway-peer",
    );
    let mut peer_app = testing_3::TestGameApp::new(
        peer.port(),
        &wasm,
        &params,
        Keypair::generate_ed25519(),
        "joining-peer",
    );

    gateway_app
        .wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("gateway app should observe both roster entries: {e}"))?;
    peer_app
        .wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("peer app should observe both roster entries: {e}"))?;

    gateway_app
        .wait_for_box_count(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("gateway app should spawn one box per roster entry: {e}"))?;
    peer_app
        .wait_for_box_count(2, Duration::from_secs(60))
        .await
        .map_err(|e| format!("peer app should spawn one box per roster entry: {e}"))?;

    assert_boxes(&mut gateway_app).map_err(|e| format!("gateway box assertions failed: {e}"))?;
    assert_boxes(&mut peer_app).map_err(|e| format!("peer box assertions failed: {e}"))?;

    gateway.shutdown().await;
    peer.shutdown().await;
    Ok(())
}
