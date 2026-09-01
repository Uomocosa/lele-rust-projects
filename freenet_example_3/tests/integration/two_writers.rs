use std::time::Duration;

use freenet_example_3::testing::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_two_writers() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut writer_a = connect(node.port).await.unwrap();
    let mut writer_b = connect(node.port).await.unwrap();
    let mut verifier = connect(node.port).await.unwrap();
    let key = deploy(&mut writer_a, &wasm).await.unwrap();
    subscribe(&mut writer_b, key).await.unwrap();
    subscribe(&mut verifier, key).await.unwrap();

    update_count(&mut writer_a, key, 0, 3).await.unwrap();
    recv_notification(&mut verifier, Duration::from_secs(10))
        .await
        .expect("verifier: first update notification not received");

    update_count(&mut writer_b, key, 1, 7).await.unwrap();
    recv_notification(&mut verifier, Duration::from_secs(10))
        .await
        .expect("verifier: second update notification not received");

    assert_eq!(get_count(&mut verifier, key).await.unwrap(), 10);
}
