use std::time::Duration;

use bevy_freenet::{boxes, roster};

fn entry(peer_id: &str) -> roster::PeerEntry {
    roster::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: vec![format!("/ip4/127.0.0.1/tcp/0/{peer_id}")],
        updated_at: 1,
    }
}

fn assert_boxes(app: &mut testing::TestGameApp) {
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
}

/// A second, structurally identical box-count test running concurrently with the first one.
/// Because each test joins its own private roster contract (its own `unique_params`), the
/// two tests cannot contaminate each other's exact-count assertions even when cargo runs
/// them in parallel.
#[tokio::test(flavor = "multi_thread")]
async fn two_instances_each_spawn_exactly_two_boxes_in_parallel() {
    let params = testing::unique_params();
    let gateway = testing::TestNode::start_gateway(0)
        .await
        .expect("gateway node should start");
    let peer = testing::TestNode::start_peer(gateway.public_port(), gateway.public_key_hex())
        .await
        .expect("peer node should join the gateway");
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
        .expect("gateway app should observe both roster entries");
    peer_app
        .wait_for_roster_len(2, Duration::from_secs(60))
        .await
        .expect("peer app should observe both roster entries");

    gateway_app
        .wait_for_box_count(2, Duration::from_secs(60))
        .await
        .expect("gateway app should spawn one box per roster entry");
    peer_app
        .wait_for_box_count(2, Duration::from_secs(60))
        .await
        .expect("peer app should spawn one box per roster entry");

    assert_boxes(&mut gateway_app);
    assert_boxes(&mut peer_app);

    gateway.shutdown().await;
    peer.shutdown().await;
}
