use freenet_example::testing::{TestNode, load_wasm};

#[tokio::test(flavor = "multi_thread")]
async fn example_standalone_demo_runs() {
    let node = TestNode::start().await.expect("node");
    let wasm = load_wasm();
    let mut client = freenet_example::GlobalCounterClient::connect(
        "127.0.0.1",
        node.port,
        &wasm,
        freenet_example::Role::Publish,
    )
    .await
    .expect("connect");
    let count = client.state().await.expect("state");
    assert_eq!(count, 0);
}
