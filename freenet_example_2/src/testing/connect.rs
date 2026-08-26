use std::time::Duration;

use crate::ClientError;
use crate::FreenetClient;

pub async fn connect(port: u16) -> Result<FreenetClient, ClientError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
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
