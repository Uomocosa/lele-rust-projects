use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::ClientError;

pub async fn recv_response_timeout(
    client: &mut crate::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, ClientError>> {
    tokio::time::timeout(timeout, client.recv_response())
        .await
        .ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
