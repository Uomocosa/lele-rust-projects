use crate::Role;
use crate::global_counter_client;

use global_counter_client::GlobalCounterClient;

/// # Errors
/// Returns an error if the client cannot be connected after retries.
pub async fn connect_with_retry(
    port: u16,
    wasm: &[u8],
    params: &[u8],
    tag: u64,
) -> GlobalCounterClient {
    let mut attempt = 0u64;
    loop {
        attempt = attempt.wrapping_add(1);
        match GlobalCounterClient::connect_with_tag(
            "127.0.0.1",
            port,
            wasm,
            params,
            Role::Publish,
            tag,
        )
        .await
        {
            Ok(c) => return c,
            Err(e) => {
                println!("connect attempt {attempt} failed: {e}");
                let backoff = std::cmp::min(attempt.wrapping_mul(5), 30);
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
            }
        }
    }
}

// no test_usage necessary
