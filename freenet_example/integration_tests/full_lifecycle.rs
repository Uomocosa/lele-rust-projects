use freenet_example::testing::*;

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
