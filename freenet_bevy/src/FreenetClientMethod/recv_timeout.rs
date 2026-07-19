use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

pub async fn recv_timeout(
    client: &mut crate::FreenetClient,
    timeout: Duration,
) -> Option<Result<HostResponse, crate::ClientError>> {
    tokio::time::timeout(timeout, crate::FreenetClientMethod::recv(client))
        .await
        .ok()
}
