use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use super::freenet_client_recv;

pub async fn recv_timeout(
    client: &mut crate::freenet_client::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, crate::ClientError>> {
    tokio::time::timeout(timeout, freenet_client_recv::recv(client))
        .await
        .ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
