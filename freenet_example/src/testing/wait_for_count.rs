use std::time::{Duration, Instant};

use crate::ClickerClient;

pub async fn wait_for_count(
    client: &mut ClickerClient,
    expected: u64,
    timeout: Duration,
) -> Result<u64, String> {
    let deadline = Instant::now() + timeout;
    loop {
        let count = client.state().await.map_err(|e| e.to_string())?;
        if count == expected {
            return Ok(count);
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "timed out waiting for count={expected}, got {count}"
            ));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
