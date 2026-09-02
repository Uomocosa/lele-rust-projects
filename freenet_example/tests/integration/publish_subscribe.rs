use freenet_example::testing::*;
use freenet_example::{GlobalCounterClient, Role};

#[tokio::test(flavor = "multi_thread")]
async fn test_publish_subscribe() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut pub_ = connect(node.port).await.unwrap();
    let key = deploy(&mut pub_, &wasm).await.unwrap();
    update_count_incrementally(&mut pub_, key, 0, 5)
        .await
        .unwrap();

    let mut sub = GlobalCounterClient::connect("127.0.0.1", node.port, &wasm, Role::Subscribe)
        .await
        .unwrap();
    assert_eq!(sub.state().await.unwrap(), 5);
}
