use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClickerError as Ce;

pub async fn state(client: &mut crate::ClickerClient) -> Result<u64, crate::ClickerError> {
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
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let mut pub_ = connect(node.port()).await;
        let key = deploy(&mut pub_, &wasm).await;
        let mut sub_a = connect(node.port()).await;
        assert_eq!(subscribe(&mut sub_a, key).await, 0);
        let mut sub_b = connect(node.port()).await;
        assert_eq!(subscribe(&mut sub_b, key).await, 0);
        update_count(&mut pub_, key, 5).await;
        let notif_a = recv_notification(&mut sub_a, std::time::Duration::from_secs(10)).await;
        let notif_b = recv_notification(&mut sub_b, std::time::Duration::from_secs(10)).await;
        if let Some(c) = notif_a { assert_eq!(c, 5); }
        if let Some(c) = notif_b { assert_eq!(c, 5); }
        update_count(&mut pub_, key, 10).await;
        let notif_a2 = recv_notification(&mut sub_a, std::time::Duration::from_secs(10)).await;
        let notif_b2 = recv_notification(&mut sub_b, std::time::Duration::from_secs(10)).await;
        if let Some(c) = notif_a2 { assert_eq!(c, 10); }
        if let Some(c) = notif_b2 { assert_eq!(c, 10); }
        assert_eq!(get_count(&mut sub_a, key).await, 10);
        assert_eq!(get_count(&mut sub_b, key).await, 10);
    }
}
