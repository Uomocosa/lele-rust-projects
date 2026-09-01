use crate::client_error;
use crate::freenet_client;
use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use client_error::ClientError;

const TIMEOUT_SECS: u64 = 60;

/// # Errors
/// Returns `ClientError::ResponseTimeout` if the response does not arrive in time.
pub async fn recv_response(
    client: &mut freenet_client::FreenetClient,
) -> Result<HostResponse, ClientError> {
    client
        .recv_response_timeout(Duration::from_secs(TIMEOUT_SECS))
        .await
        .ok_or(ClientError::ResponseTimeout)?
}

// no test_usage necessary — exercised via integration tests
