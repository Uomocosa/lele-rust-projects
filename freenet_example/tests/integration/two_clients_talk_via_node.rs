use std::time::Duration;

use freenet_example::testing::*;
use freenet_example::{ClickerClient, Role};

#[tokio::test(flavor = "multi_thread")]
async fn test_two_clients_talk_via_node() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();

    let mut pub_ = ClickerClient::connect("127.0.0.1", node.port, &wasm, Role::Publish)
        .await
        .unwrap();
    assert_eq!(pub_.tick().await.unwrap(), 1);

    let mut sub = ClickerClient::connect("127.0.0.1", node.port, &wasm, Role::Subscribe)
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
