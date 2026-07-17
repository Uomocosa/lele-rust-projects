use std::time::Duration;

use freenet_example::testing::*;
use freenet_example::{ClickerClient, Role};

#[tokio::test(flavor = "multi_thread")]
async fn test_full_lifecycle() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut client = connect(node.port()).await.unwrap();
    let key = deploy(&mut client, &wasm).await.unwrap();

    assert_eq!(get_count(&mut client, key).await.unwrap(), 0);

    update_count(&mut client, key, 42).await.unwrap();
    assert_eq!(get_count(&mut client, key).await.unwrap(), 42);

    update_count(&mut client, key, 99).await.unwrap();
    assert_eq!(get_count(&mut client, key).await.unwrap(), 99);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_persistence() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let key;
    {
        let mut client = connect(node.port()).await.unwrap();
        key = deploy(&mut client, &wasm).await.unwrap();
        update_count(&mut client, key, 5).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 5);
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    {
        let mut client = connect(node.port()).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 5);
        update_count(&mut client, key, 8).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 8);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_publish_subscribe() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut pub_ = connect(node.port()).await.unwrap();
    let key = deploy(&mut pub_, &wasm).await.unwrap();
    update_count(&mut pub_, key, 5).await.unwrap();

    let mut sub = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe)
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 5);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_standalone_demo() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut clicker = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
        .await
        .unwrap();
    assert!(clicker.count() == 0);
    for expected in 1..=3 {
        assert_eq!(clicker.tick().await.unwrap(), expected);
    }
    assert_eq!(clicker.state().await.unwrap(), 3);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_two_writers() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut writer_a = connect(node.port()).await.unwrap();
    let mut writer_b = connect(node.port()).await.unwrap();
    let mut verifier = connect(node.port()).await.unwrap();
    let key = deploy(&mut writer_a, &wasm).await.unwrap();
    subscribe(&mut writer_b, key).await.unwrap();
    subscribe(&mut verifier, key).await.unwrap();

    update_count(&mut writer_a, key, 3).await.unwrap();
    recv_notification(&mut verifier, Duration::from_secs(10))
        .await
        .expect("verifier: first update notification not received");

    update_count(&mut writer_b, key, 7).await.unwrap();
    recv_notification(&mut verifier, Duration::from_secs(10))
        .await
        .expect("verifier: second update notification not received");

    assert_eq!(get_count(&mut verifier, key).await.unwrap(), 7);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_subscriber_notifications() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut pub_ = connect(node.port()).await.unwrap();
    let key = deploy(&mut pub_, &wasm).await.unwrap();

    let mut sub_a = connect(node.port()).await.unwrap();
    assert_eq!(subscribe(&mut sub_a, key).await.unwrap(), 0);
    let mut sub_b = connect(node.port()).await.unwrap();
    assert_eq!(subscribe(&mut sub_b, key).await.unwrap(), 0);

    update_count(&mut pub_, key, 5).await.unwrap();
    let notif_a = recv_notification(&mut sub_a, Duration::from_secs(10))
        .await
        .expect("sub_a: update notification not received");
    let notif_b = recv_notification(&mut sub_b, Duration::from_secs(10))
        .await
        .expect("sub_b: update notification not received");
    assert_eq!(notif_a, 5);
    assert_eq!(notif_b, 5);

    update_count(&mut pub_, key, 10).await.unwrap();
    let notif_a2 = recv_notification(&mut sub_a, Duration::from_secs(10))
        .await
        .expect("sub_a: second update notification not received");
    let notif_b2 = recv_notification(&mut sub_b, Duration::from_secs(10))
        .await
        .expect("sub_b: second update notification not received");
    assert_eq!(notif_a2, 10);
    assert_eq!(notif_b2, 10);

    assert_eq!(get_count(&mut sub_a, key).await.unwrap(), 10);
    assert_eq!(get_count(&mut sub_b, key).await.unwrap(), 10);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_two_clients_talk_via_node() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();

    let mut pub_ = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
        .await
        .unwrap();
    assert_eq!(pub_.tick().await.unwrap(), 1);

    let mut sub = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe)
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 1);

    assert_eq!(sub.tick().await.unwrap(), 2);
    wait_for_count(&mut pub_, 2, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(pub_.state().await.unwrap(), 2);

    assert_eq!(pub_.tick().await.unwrap(), 3);
    wait_for_count(&mut sub, 3, Duration::from_secs(10))
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 3);
}
