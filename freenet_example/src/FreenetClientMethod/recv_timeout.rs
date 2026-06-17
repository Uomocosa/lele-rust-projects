use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::ClientError;

pub async fn recv_timeout(
    client: &mut crate::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, ClientError>> {
    tokio::time::timeout(timeout, client.recv()).await.ok()
}

#[cfg(test)]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await;
        let mut client = connect(node.port()).await;
        let result = client.recv_timeout(std::time::Duration::from_millis(10)).await;
        assert!(result.is_none());
    }
}
