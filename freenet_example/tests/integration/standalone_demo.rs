use freenet_example::testing::*;
use freenet_example::{GlobalCounterClient, Role};

#[tokio::test(flavor = "multi_thread")]
async fn test_standalone_demo() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut global_counter =
        GlobalCounterClient::connect("127.0.0.1", node.port, &wasm, Role::Publish)
            .await
            .unwrap();
    assert_eq!(global_counter.count(), 0);
    for expected in 1..=3 {
        assert_eq!(global_counter.tick().await.unwrap(), expected);
    }
    assert_eq!(global_counter.state().await.unwrap(), 3);
}
