use crate::clicker_client;
use crate::clicker_error;
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use clicker_error::ClickerError as Ce;

pub async fn state(
    client: &mut clicker_client::ClickerClient,
) -> Result<u64, clicker_error::ClickerError> {
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
            let count = bincode::deserialize(state.as_ref()).unwrap_or(0);
            client.count = count;
            Ok(count)
        }
        other => Err(Ce::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let mut pub_ = connect(node.port()).await.unwrap();
        let key = deploy(&mut pub_, &wasm).await.unwrap();
        let mut sub_a = connect(node.port()).await.unwrap();
        assert_eq!(subscribe(&mut sub_a, key).await.unwrap(), 0);
        let mut sub_b = connect(node.port()).await.unwrap();
        assert_eq!(subscribe(&mut sub_b, key).await.unwrap(), 0);
        update_count(&mut pub_, key, 5).await.unwrap();
        let notif_a = recv_notification(&mut sub_a, std::time::Duration::from_secs(10))
            .await
            .expect("sub_a: update notification not received");
        let notif_b = recv_notification(&mut sub_b, std::time::Duration::from_secs(10))
            .await
            .expect("sub_b: update notification not received");
        assert_eq!(notif_a, 5);
        assert_eq!(notif_b, 5);
        update_count(&mut pub_, key, 10).await.unwrap();
        let notif_a2 = recv_notification(&mut sub_a, std::time::Duration::from_secs(10))
            .await
            .expect("sub_a: second update notification not received");
        let notif_b2 = recv_notification(&mut sub_b, std::time::Duration::from_secs(10))
            .await
            .expect("sub_b: second update notification not received");
        assert_eq!(notif_a2, 10);
        assert_eq!(notif_b2, 10);
        assert_eq!(get_count(&mut sub_a, key).await.unwrap(), 10);
        assert_eq!(get_count(&mut sub_b, key).await.unwrap(), 10);
    }
}
