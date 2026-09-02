use std::time::Duration;

use freenet_example::testing::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_subscriber_notifications() {
    let node = TestNode::start().await.unwrap();
    let wasm = load_wasm();
    let mut publisher = connect(node.port).await.unwrap();
    let key = deploy(&mut publisher, &wasm).await.unwrap();

    let mut subscriber_alpha = connect(node.port).await.unwrap();
    assert_eq!(subscribe(&mut subscriber_alpha, key).await.unwrap(), 0);
    let mut subscriber_beta = connect(node.port).await.unwrap();
    assert_eq!(subscribe(&mut subscriber_beta, key).await.unwrap(), 0);

    update_count(&mut publisher, key, 0, 5).await.unwrap();
    let notification_alpha = recv_notification(&mut subscriber_alpha, Duration::from_secs(10))
        .await
        .expect("subscriber_alpha: update notification not received");
    let notification_beta = recv_notification(&mut subscriber_beta, Duration::from_secs(10))
        .await
        .expect("subscriber_beta: update notification not received");
    assert_eq!(notification_alpha, 5);
    assert_eq!(notification_beta, 5);

    update_count(&mut publisher, key, 0, 10).await.unwrap();
    let second_alpha = recv_notification(&mut subscriber_alpha, Duration::from_secs(10))
        .await
        .expect("subscriber_alpha: second update notification not received");
    let second_beta = recv_notification(&mut subscriber_beta, Duration::from_secs(10))
        .await
        .expect("subscriber_beta: second update notification not received");
    assert_eq!(second_alpha, 10);
    assert_eq!(second_beta, 10);

    assert_eq!(get_count(&mut subscriber_alpha, key).await.unwrap(), 10);
    assert_eq!(get_count(&mut subscriber_beta, key).await.unwrap(), 10);
}
