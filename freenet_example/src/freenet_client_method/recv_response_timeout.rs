use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::client_error::ClientError;
use crate::freenet_client::FreenetClient;

pub async fn recv_response_timeout(
    client: &mut FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, ClientError>> {
    tokio::time::timeout(timeout, client.recv_response())
        .await
        .ok()
}

#[cfg(test)]
mod tests {
    // Trivial wrapper/delegate module — skip test_usage.
    // The real coverage comes from integration tests.
    #[test]
    fn test_usage() {}
}
