use freenet_example_2::testing::*;
use freenet_example_2::{ClickerClient, Role};

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
