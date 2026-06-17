use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClickerError as Ce;

pub async fn tick(client: &mut crate::ClickerClient) -> Result<u64, crate::ClickerError> {
    while let Some(result) = client.client.recv_timeout(Duration::from_millis(10)).await {
        if let HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update, ..
        }) = result?
        {
            client.count = match &update {
                UpdateData::State(s) => bincode::deserialize(s.as_ref()).unwrap_or(0),
                UpdateData::Delta(d) => bincode::deserialize(d.as_ref()).unwrap_or(0),
                _ => 0,
            };
        }
    }

    client.count = client.count.wrapping_add(1);
    let new_state = State::from(bincode::serialize(&client.count)?);
    let update_req = ContractRequest::Update {
        key: client.contract_key,
        data: UpdateData::State(new_state),
    };
    client
        .client
        .send(ClientRequest::ContractOp(update_req))
        .await?;

    match client.client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
        other => return Err(Ce::UnexpectedResponse(format!("{other:?}"))),
    }

    Ok(client.count)
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let mut client = connect(node.port()).await;
        let key = deploy(&mut client, &wasm).await;
        for expected in 1..=5 {
            update_count(&mut client, key, expected).await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        assert_eq!(get_count(&mut client, key).await, 5);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_two_publishers() {
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let mut writer_a = connect(node.port()).await;
        let mut writer_b = connect(node.port()).await;
        let mut verifier = connect(node.port()).await;
        let key = deploy(&mut writer_a, &wasm).await;
        subscribe(&mut writer_b, key).await;
        subscribe(&mut verifier, key).await;
        update_count(&mut writer_a, key, 3).await;
        recv_notification(&mut verifier, std::time::Duration::from_secs(10)).await;
        update_count(&mut writer_b, key, 7).await;
        recv_notification(&mut verifier, std::time::Duration::from_secs(10)).await;
        assert_eq!(get_count(&mut verifier, key).await, 7);
    }
}
