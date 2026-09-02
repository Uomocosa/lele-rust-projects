use crate::global_counter_client;
use crate::global_counter_error;
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use global_counter_error::GlobalCounterError as Ce;

/// # Errors
/// Returns `GlobalCounterError` if the get request fails or the response is unexpected.
pub async fn state(
    client: &mut global_counter_client::GlobalCounterClient,
) -> Result<u64, global_counter_error::GlobalCounterError> {
    let get_req = ContractRequest::Get {
        key: *client.contract_key.id(),
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client
        .client
        .send(ClientRequest::ContractOp(get_req))
        .await?;
    match client.client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
            client.slots = bincode::deserialize(state.as_ref()).unwrap_or_default();
            Ok(client.slots.values().sum())
        }
        other => Err(Ce::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let mut publisher = connect(node.port).await.unwrap();
        let key = deploy(&mut publisher, &wasm).await.unwrap();
        let mut subscriber_alpha = connect(node.port).await.unwrap();
        assert_eq!(subscribe(&mut subscriber_alpha, key).await.unwrap(), 0);
        let mut subscriber_beta = connect(node.port).await.unwrap();
        assert_eq!(subscribe(&mut subscriber_beta, key).await.unwrap(), 0);
        update_count_incrementally(&mut publisher, key, 0, 5)
            .await
            .unwrap();
        let mut notification_alpha = 0;
        for _ in 0..5 {
            notification_alpha =
                recv_notification(&mut subscriber_alpha, std::time::Duration::from_secs(10))
                    .await
                    .expect("subscriber_alpha: update notification not received");
        }
        let mut notification_beta = 0;
        for _ in 0..5 {
            notification_beta =
                recv_notification(&mut subscriber_beta, std::time::Duration::from_secs(10))
                    .await
                    .expect("subscriber_beta: update notification not received");
        }
        assert_eq!(notification_alpha, 5);
        assert_eq!(notification_beta, 5);
        update_count_incrementally(&mut publisher, key, 0, 10)
            .await
            .unwrap();
        let mut second_alpha = 0;
        for _ in 0..5 {
            second_alpha =
                recv_notification(&mut subscriber_alpha, std::time::Duration::from_secs(10))
                    .await
                    .expect("subscriber_alpha: second update notification not received");
        }
        let mut second_beta = 0;
        for _ in 0..5 {
            second_beta =
                recv_notification(&mut subscriber_beta, std::time::Duration::from_secs(10))
                    .await
                    .expect("subscriber_beta: second update notification not received");
        }
        assert_eq!(second_alpha, 10);
        assert_eq!(second_beta, 10);
        assert_eq!(get_count(&mut subscriber_alpha, key).await.unwrap(), 10);
        assert_eq!(get_count(&mut subscriber_beta, key).await.unwrap(), 10);
        drop(publisher);
    }
}
