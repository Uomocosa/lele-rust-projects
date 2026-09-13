use freenet_stdlib::client_api::HostResponse;

use crate::discovery;

pub async fn recv_response_timeout(
    client: &mut discovery::Client,
    timeout: std::time::Duration,
) -> Option<Result<HostResponse, discovery::Error>> {
    tokio::time::timeout(timeout, client.recv_response())
        .await
        .ok()
}
// no test_usage necessary
