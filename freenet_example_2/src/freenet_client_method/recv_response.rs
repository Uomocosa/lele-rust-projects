use crate::client_error;
use crate::freenet_client;
use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use client_error::ClientError;

pub async fn recv_response(
    client: &mut freenet_client::FreenetClient,
) -> Result<HostResponse, ClientError> {
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
        let node = TestNode::start().await.unwrap();
        let mut client = connect(node.port()).await.unwrap();
        let wasm = load_wasm();
        let key = deploy(&mut client, &wasm).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 0);
    }
}
