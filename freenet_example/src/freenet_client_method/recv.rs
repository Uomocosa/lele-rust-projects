use freenet_stdlib::client_api::HostResponse;

use crate::client_error::ClientError as Ce;
use crate::freenet_client::FreenetClient;

/// # Errors
/// Returns `ClientError` if the channel is disconnected or the node returns an error.
pub async fn recv(client: &mut FreenetClient) -> Result<HostResponse, Ce> {
    match client.read.recv().await {
        Some(Ok(response)) => Ok(response),
        Some(Err(e)) => Err(Ce::from(e)),
        None => Err(Ce::Disconnected),
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
        let count = get_count(&mut client, key).await.unwrap();
        assert_eq!(count, 0);
    }
}
