use std::time::Duration;

use crate::ClientError;
use crate::FreenetClient;

/// # Errors
/// Returns `ClientError::DeadlineOverflow` if the deadline overflows or `ClientError::ConnectionTimeout` if the node is unreachable.
pub async fn connect(port: u16) -> Result<FreenetClient, ClientError> {
    let Some(deadline) = tokio::time::Instant::now().checked_add(Duration::from_secs(15)) else {
        return Err(ClientError::DeadlineOverflow);
    };
    loop {
        match FreenetClient::connect("127.0.0.1", port).await {
            Ok(c) => return Ok(c),
            Err(_e) => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(ClientError::ConnectionTimeout);
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

// no test_usage necessary — exercised via integration tests
