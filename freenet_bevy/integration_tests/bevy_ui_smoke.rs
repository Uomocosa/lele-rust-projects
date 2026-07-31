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
