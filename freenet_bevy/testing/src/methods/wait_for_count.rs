use std::time::Duration;

use freenet_stdlib::prelude::ContractKey;

use crate::methods::get_count;

pub async fn wait_for_count(
    client: &mut freenet_bevy::FreenetClient,
    key: ContractKey,
    expected: u64,
    timeout: Duration,
) -> Result<(), freenet_bevy::ClientError> {
    let deadline = tokio::time::Instant::now() + timeout;
    while tokio::time::Instant::now() < deadline {
        if get_count(client, key).await? == expected {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err(freenet_bevy::ClientError::ResponseTimeout)
}
