use std::time::Duration;

use freenet_example::testing::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_persistence() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let key;
    {
        let mut client = connect(node.port).await.unwrap();
        key = deploy(&mut client, &wasm).await.unwrap();
        update_count(&mut client, key, 0, 5).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 5);
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
    {
        let mut client = connect(node.port).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 5);
        update_count(&mut client, key, 0, 8).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 8);
    }
}
