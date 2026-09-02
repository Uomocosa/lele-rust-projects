use std::time::{Duration, Instant};

use crate::GlobalCounterClient;

/// # Errors
/// Returns `Err` if the deadline overflows or the count does not reach `expected` within `timeout`.
pub async fn wait_for_count(
    client: &mut GlobalCounterClient,
    expected: u64,
    timeout: Duration,
) -> Result<u64, String> {
    let Some(deadline) = Instant::now().checked_add(timeout) else {
        return Err(format!("deadline overflow for timeout {timeout:?}"));
    };
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

// no test_usage necessary — exercised via integration tests
