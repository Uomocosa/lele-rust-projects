use crate::client_error;
use crate::freenet_client;
use std::time::Duration;

use freenet_stdlib::client_api::HostResponse;

use client_error::ClientError;

const TIMEOUT_SECS: u64 = 60;

pub(crate) async fn recv_response(
    client: &mut freenet_client::FreenetClient,
) -> Result<HostResponse, ClientError> {
    match client
        .recv_response_timeout(Duration::from_secs(TIMEOUT_SECS))
        .await
    {
        Some(result) => result,
        None => Err(ClientError::ResponseTimeout),
    }
}

// no test_usage necessary — exercised via integration tests
