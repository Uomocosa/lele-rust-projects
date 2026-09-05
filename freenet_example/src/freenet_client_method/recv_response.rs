use freenet_stdlib::client_api::{ContractResponse, HostResponse};

use crate::client_error::ClientError;
use crate::freenet_client::FreenetClient;

/// # Errors
/// Returns `ClientError` if the channel is disconnected or the node returns an error.
pub async fn recv_response(client: &mut FreenetClient) -> Result<HostResponse, ClientError> {
    loop {
        match client.recv().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {}
            other => return Ok(other),
        }
    }
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let mut client = connect(node.port).await.unwrap();
        let wasm = load_wasm();
        let key = deploy(&mut client, &wasm).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 0);
    }
}
