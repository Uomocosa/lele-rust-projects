use std::time::Duration;

use freenet_libp2p_bevy_example_1_lib::{boxes, roster};

// needed helper:
fn entry(peer_id: &str) -> roster::PeerEntry {
    roster::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: vec![format!("/ip4/127.0.0.1/tcp/0/{peer_id}")],
        updated_at: 1,
    }
}

// needed helper:
fn assert_boxes(app: &mut testing::TestGameApp) -> Result<(), String> {
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
    assert!(
        (remote.1.y - boxes::SPAWN_Y).abs() < f32::EPSILON,
        "remote kinematic box must stay at spawn height"
    );

    let mut xs: Vec<f32> = spawns.iter().map(|(_, pos, _)| pos.x).collect();
    xs.sort_by(f32::total_cmp);
    assert!(xs[0] != xs[1], "box x positions must be distinct");
    let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;
    assert!(
        xs[0] >= -bound && xs[1] <= bound,
        "box x positions must stay on the ground"
    );
    Ok(())
}

/// A second, structurally identical box-count harness running after the first one. Each run joins
/// its own private roster contract (its own `unique_params`), so runs cannot contaminate each
/// other's exact-count assertions.
#[tokio::test(flavor = "multi_thread")]
async fn two_node_box_count_parallel() -> Result<(), Box<dyn std::error::Error>> {
    let params = testing::unique_params();
    let gateway = testing::TestNode::start_gateway(0)
        .await
        .map_err(|e| format!("gateway node should start: {e}"))?;
    let peer = testing::TestNode::start_peer(gateway.public_port(), gateway.public_key_hex())
        .await
        .map_err(|e| format!("peer node should join the gateway: {e}"))?;
    let wasm = testing::load_wasm();

    let mut gateway_app = testing::TestGameApp::new(
        gateway.port(),
        &wasm,
        &params,
        boxes::PlayerId(3),
        entry("gateway-peer"),
    );
    let mut peer_app = testing::TestGameApp::new(
        peer.port(),
        &wasm,
        &params,
        boxes::PlayerId(4),
        entry("joining-peer"),
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
