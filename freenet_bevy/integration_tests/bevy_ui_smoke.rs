use std::time::Duration;

use tokio::sync::mpsc;

use freenet_bevy::clicker::{ClickerCommand, ClickerEvent};
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
    let (_cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<ClickerCommand>();
    let (evt_tx, mut evt_rx) = mpsc::unbounded_channel::<ClickerEvent>();

    evt_tx
        .send(ClickerEvent::Notification { count: 99 })
        .unwrap();

    let event = evt_rx.recv().await.unwrap();
    match event {
        ClickerEvent::Notification { count } => assert_eq!(count, 99),
        _ => unreachable!(),
    }
}
