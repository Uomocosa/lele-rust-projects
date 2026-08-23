use std::time::Duration;

/// The M2 checkpoint: two separate embedded nodes join the same Freenet network (node B dials
/// node A as its gateway), each deploys/merges its own identity-keyed, signed roster entry, and
/// each ends up observing a 2-entry roster — proving the commutative-merge contract actually
/// propagates state across a real join, not just within a single isolated node, and that the
/// client's ed25519 signatures validate against the contract.
#[tokio::test(flavor = "multi_thread")]
async fn two_node_roster() -> Result<(), Box<dyn std::error::Error>> {
    let params = testing_3::unique_params();
    let gateway = testing_3::TestNode::start_gateway(0)
        .await
        .map_err(|e| format!("gateway node should start: {e}"))?;

    let peer = testing_3::TestNode::start_peer(gateway.public_port(), gateway.public_key_hex())
        .await
        .map_err(|e| format!("peer node should join the gateway: {e}"))?;

    let wasm = testing_3::load_wasm();

    let (_, gateway_id, gateway_entry) = testing_3::new_identity("gateway-peer");
    let (_, peer_id, peer_entry) = testing_3::new_identity("joining-peer");

    let (mut gateway_client, gateway_roster) =
        testing_3::deploy_roster(gateway.port(), &wasm, &params, gateway_id, gateway_entry)
            .await
            .map_err(|e| format!("gateway should deploy/join the roster contract: {e}"))?;
    if !gateway_roster.contains_key(&gateway_id) {
        return Err("gateway roster should contain its own player id".into());
    }

    let (peer_client, peer_roster) =
        testing_3::deploy_roster(peer.port(), &wasm, &params, peer_id, peer_entry)
            .await
            .map_err(|e| format!("peer should deploy/join the roster contract: {e}"))?;
    drop(peer_client);

    let gateway_view =
        testing_3::wait_for_roster_len(&mut gateway_client, 2, Duration::from_secs(60))
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
    if !gateway_view.contains_key(&gateway_id) {
        return Err("gateway view should contain its own id".into());
    }
    if !gateway_view.contains_key(&peer_id) {
        return Err("gateway view should contain the peer's id".into());
    }
    if !peer_roster.contains_key(&gateway_id) {
        return Err("peer roster should contain the gateway's id".into());
    }
    if !peer_roster.contains_key(&peer_id) {
        return Err("peer roster should contain its own id".into());
    }

    gateway.shutdown().await;
    peer.shutdown().await;
    Ok(())
}
