use std::time::Duration;

use bevy_freenet::{boxes, roster};

fn entry(peer_id: &str) -> roster::PeerEntry {
    roster::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: vec![format!("/ip4/127.0.0.1/tcp/0/{peer_id}")],
        updated_at: 1,
    }
}

/// The M2 checkpoint: two separate embedded nodes join the same Freenet network (node B
/// dials node A as its gateway), each Puts/merges its own roster entry, and each ends up
/// observing a 2-entry roster — proving the commutative-merge contract actually propagates
/// state across a real join, not just within a single isolated node.
#[tokio::test(flavor = "multi_thread")]
async fn two_nodes_see_each_other_in_the_roster() {
    let gateway = testing::TestNode::start_gateway(0)
        .await
        .expect("gateway node should start");

    let peer = testing::TestNode::start_peer(gateway.public_port(), gateway.public_key_hex())
        .await
        .expect("peer node should join the gateway");

    let wasm = testing::load_wasm();

    let (mut gateway_client, gateway_roster) = testing::deploy_roster(
        gateway.port(),
        &wasm,
        boxes::PlayerId { value: 1 },
        entry("gateway-peer"),
    )
    .await
    .expect("gateway should deploy/join the roster contract");
    assert!(gateway_roster.contains_key(&boxes::PlayerId { value: 1 }));

    // The peer's own `deploy_roster` call already merges in the gateway's existing entry
    // synchronously (it Gets the contract, sees entry 1, merges in entry 2, and Updates) —
    // it does not need to wait for a push notification of its own write.
    let (peer_client, peer_roster) = testing::deploy_roster(
        peer.port(),
        &wasm,
        boxes::PlayerId { value: 2 },
        entry("joining-peer"),
    )
    .await
    .expect("peer should deploy/join the roster contract");
    drop(peer_client);

    // The gateway only learns about the peer's entry via a push notification, since its
    // own client already returned before the peer's write happened.
    let gateway_view =
        testing::wait_for_roster_len(&mut gateway_client, 2, Duration::from_secs(30))
            .await
            .expect("gateway should observe both roster entries");

    assert_eq!(gateway_view.len(), 2);
    assert_eq!(peer_roster.len(), 2);
    assert!(gateway_view.contains_key(&boxes::PlayerId { value: 1 }));
    assert!(gateway_view.contains_key(&boxes::PlayerId { value: 2 }));
    assert!(peer_roster.contains_key(&boxes::PlayerId { value: 1 }));
    assert!(peer_roster.contains_key(&boxes::PlayerId { value: 2 }));

    gateway.shutdown().await;
    peer.shutdown().await;
}
