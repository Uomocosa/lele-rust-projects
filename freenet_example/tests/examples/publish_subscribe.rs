use freenet_example_3::testing::{
    TestNode, connect, deploy, get_count, load_wasm, subscribe, update_count,
};

#[tokio::test(flavor = "multi_thread")]
async fn example_publish_subscribe_runs() {
    let node = TestNode::start().await.expect("node");
    let wasm = load_wasm();
    let mut publisher = connect(node.port).await.expect("connect");
    let key = deploy(&mut publisher, &wasm).await.expect("deploy");
    let mut subscriber = connect(node.port).await.expect("connect sub");
    assert_eq!(subscribe(&mut subscriber, key).await.expect("subscribe"), 0);
    update_count(&mut publisher, key, 0, 5)
        .await
        .expect("update");
    assert_eq!(get_count(&mut subscriber, key).await.expect("get"), 5);
}
