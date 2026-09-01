use crate::client_error;
use crate::freenet_client;
use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use client_error::ClientError;

pub async fn recv_timeout(
    client: &mut freenet_client::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, ClientError>> {
    tokio::time::timeout(timeout, client.recv()).await.ok()
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let mut client = connect(node.port).await.unwrap();
        let result = client
            .recv_timeout(std::time::Duration::from_millis(10))
            .await;
        assert!(result.is_none());
    }
}
