use std::time::Duration;

use crate::structs::client_error::ClientError;

pub async fn connect(
    port: u16,
) -> Result<crate::structs::freenet_client::FreenetClient, ClientError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match crate::structs::freenet_client::FreenetClient::connect("127.0.0.1", port).await {
            Ok(client) => return Ok(client),
            Err(e) if tokio::time::Instant::now() > deadline => return Err(e),
            _ => tokio::time::sleep(Duration::from_millis(200)).await,
        }
    }
}
