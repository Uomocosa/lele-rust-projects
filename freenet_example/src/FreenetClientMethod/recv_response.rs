use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::ClientError;

pub async fn recv_response(client: &mut crate::FreenetClient) -> Result<HostResponse, ClientError> {
    loop {
        match client.recv().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => continue,
            other => return Ok(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await;
        let mut client = connect(node.port()).await;
        let wasm = load_wasm();
        let key = deploy(&mut client, &wasm).await;
        assert_eq!(get_count(&mut client, key).await, 0);
    }
}
