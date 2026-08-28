use std::time::Duration;

use freenet_example_2::testing::*;

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

    update_count(&mut pub_, key, 0, 5).await.unwrap();
    let notif_a = recv_notification(&mut sub_a, Duration::from_secs(10))
        .await
        .expect("sub_a: update notification not received");
    let notif_b = recv_notification(&mut sub_b, Duration::from_secs(10))
        .await
        .expect("sub_b: update notification not received");
    assert_eq!(notif_a, 5);
    assert_eq!(notif_b, 5);

    update_count(&mut pub_, key, 0, 10).await.unwrap();
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
