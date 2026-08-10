use std::time::Duration;

use tokio::sync::mpsc;

use freenet_bevy::clicker::{Command, Event};
use testing::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_deploy_and_count() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut client = connect(node.port()).await.unwrap();
    let key = deploy(&mut client, &wasm).await.unwrap();

    assert_eq!(get_count(&mut client, key).await.unwrap(), 0);

    update_count(&mut client, key, 42).await.unwrap();
    assert_eq!(get_count(&mut client, key).await.unwrap(), 42);

    wait_for_count(&mut client, key, 42, Duration::from_secs(5))
        .await
        .unwrap();
}

/// Waits for a notification carrying `expected`, ignoring any stale/self-echo notifications
/// (e.g. a client's own update bouncing back) that might be queued ahead of it.
async fn expect_notification(
    client: &mut freenet_bevy::freenet::FreenetClient,
    expected: u64,
    timeout: Duration,
) -> bool {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if recv_notification(client, Duration::from_millis(500)).await == Some(expected) {
            return true;
        }
    }
    false
}

#[tokio::test(flavor = "multi_thread")]
async fn test_two_clients_see_each_others_updates() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();

    let mut client_a = connect(node.port()).await.unwrap();
    let key = deploy(&mut client_a, &wasm).await.unwrap();

    let mut client_b = connect(node.port()).await.unwrap();
    subscribe(&mut client_b, key).await.unwrap();

    update_count(&mut client_a, key, 7).await.unwrap();
    assert!(
        expect_notification(&mut client_b, 7, Duration::from_secs(5)).await,
        "client B did not receive client A's update"
    );

    update_count(&mut client_b, key, 13).await.unwrap();
    assert!(
        expect_notification(&mut client_a, 13, Duration::from_secs(5)).await,
        "client A did not receive client B's update"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_state_persists_across_node_restart() {
    let dir = tempfile::tempdir().unwrap();
    let wasm = load_wasm();

    let node = TestNode::start_at(dir.path()).await.unwrap();
    let mut client = connect(node.port()).await.unwrap();
    let key = deploy(&mut client, &wasm).await.unwrap();
    update_count(&mut client, key, 7).await.unwrap();
    wait_for_count(&mut client, key, 7, Duration::from_secs(5))
        .await
        .unwrap();
    drop(client);
    node.shutdown().await; // simulates the app closing

    let node = TestNode::start_at(dir.path()).await.unwrap();
    let mut client = connect(node.port()).await.unwrap();
    let count = subscribe(&mut client, key).await.unwrap();
    assert_eq!(count, 7, "restarted node lost the persisted count");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_channel_roundtrip() {
    let (_cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let (evt_tx, mut evt_rx) = mpsc::unbounded_channel::<Event>();

    evt_tx.send(Event::Notification { count: 99 }).unwrap();

    let event = evt_rx.recv().await.unwrap();
    match event {
        Event::Notification { count } => assert_eq!(count, 99),
        _ => unreachable!(),
    }
}
