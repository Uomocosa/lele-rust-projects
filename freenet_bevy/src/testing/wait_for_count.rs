use std::time::Duration;

use freenet_stdlib::prelude::ContractKey;

use crate::ClientError;
use crate::FreenetClient;

pub async fn wait_for_count(
    client: &mut FreenetClient,
    key: ContractKey,
    expected: u64,
    timeout: Duration,
) -> Result<(), ClientError> {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if crate::testing::get_count(client, key).await? == expected {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(ClientError::ResponseTimeout)
}
