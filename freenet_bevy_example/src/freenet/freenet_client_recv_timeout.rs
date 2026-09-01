use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use super::freenet_client_recv;
use crate::freenet;

pub async fn recv_timeout(
    client: &mut freenet::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, freenet::FreenetConnectionError>> {
    tokio::time::timeout(timeout, freenet_client_recv::recv(client))
        .await
        .ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
