use std::time::Duration;

use freenet_libp2p_bevy_example_1_lib::{boxes, roster};

fn entry(peer_id: &str) -> roster::PeerEntry {
    roster::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: vec![format!("/ip4/127.0.0.1/tcp/0/{peer_id}")],
        updated_at: 1,
    }
}

/// The M2 checkpoint: two separate embedded nodes join the same Freenet network (node B dials
/// node A as its gateway), each Puts/merges its own roster entry, and each ends up observing a
/// 2-entry roster — proving the commutative-merge contract actually propagates state across a real
/// join, not just within a single isolated node.
///
/// This is a `[[bin]]` (not an integration test) so cargo emits it at a stable, un-hashed path,
/// letting Windows Firewall rules keyed to that path stay valid across dependency changes.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let params = testing::unique_params();
    let gateway = testing::TestNode::start_gateway(0)
        .await
        .map_err(|e| format!("gateway node should start: {e}"))?;

    let peer = testing::TestNode::start_peer(gateway.public_port(), gateway.public_key_hex())
        .await
        .map_err(|e| format!("peer node should join the gateway: {e}"))?;

    let wasm = testing::load_wasm();

    let (mut gateway_client, gateway_roster) = testing::deploy_roster(
        gateway.port(),
        &wasm,
        &params,
        boxes::PlayerId(1),
        entry("gateway-peer"),
    )
    .await
    .map_err(|e| format!("gateway should deploy/join the roster contract: {e}"))?;
    if !gateway_roster.contains_key(&boxes::PlayerId(1)) {
        return Err("gateway roster should contain its own player id".into());
    }

    // The peer's own `deploy_roster` call already merges in the gateway's existing entry
    // synchronously (it Gets the contract, sees entry 1, merges in entry 2, and Updates) — it
    // does not need to wait for a push notification of its own write.
    let (peer_client, peer_roster) = testing::deploy_roster(
        peer.port(),
        &wasm,
        &params,
        boxes::PlayerId(2),
        entry("joining-peer"),
    )
    .await
    .map_err(|e| format!("peer should deploy/join the roster contract: {e}"))?;
    drop(peer_client);

    // The gateway only learns about the peer's entry via a push notification, since its own
    // client already returned before the peer's write happened.
    let gateway_view =
        testing::wait_for_roster_len(&mut gateway_client, 2, Duration::from_secs(60))
            .await
            .ok_or("gateway should observe both roster entries within timeout")?;

    if gateway_view.len() != 2 {
        return Err(format!(
            "gateway view should have 2 entries, got {}",
            gateway_view.len()
        )
        .into());
    }
    if peer_roster.len() != 2 {
        return Err(format!(
            "peer roster should have 2 entries, got {}",
            peer_roster.len()
        )
        .into());
    }
    if !gateway_view.contains_key(&boxes::PlayerId(1)) {
        return Err("gateway view should contain player 1".into());
    }
    if !gateway_view.contains_key(&boxes::PlayerId(2)) {
        return Err("gateway view should contain player 2".into());
    }
    if !peer_roster.contains_key(&boxes::PlayerId(1)) {
        return Err("peer roster should contain player 1".into());
    }
    if !peer_roster.contains_key(&boxes::PlayerId(2)) {
        return Err("peer roster should contain player 2".into());
    }

    gateway.shutdown().await;
    peer.shutdown().await;
    Ok(())
}

// no test_usage necessary
