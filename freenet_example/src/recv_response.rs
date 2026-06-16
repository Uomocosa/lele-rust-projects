use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use crate::ClientError;

const TIMEOUT_SECS: u64 = 10;

pub(crate) async fn recv_response(
    client: &mut crate::FreenetClient,
) -> Result<HostResponse, ClientError> {
    match client
        .recv_response_timeout(Duration::from_secs(TIMEOUT_SECS))
        .await
    {
        Some(result) => result,
        None => Err(ClientError::ResponseTimeout),
    }
}
