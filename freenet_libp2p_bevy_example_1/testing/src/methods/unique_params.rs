use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic, run-unique `Parameters` payload per call. Every test run deploys to a fresh
/// roster contract key instead of reusing `roster-test-0`, because roster contracts persist on
/// the mainnet and stale entries from a prior run would pollute the next run's assertions.
pub fn unique_params() -> Vec<u8> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let run = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!(
        "roster-test-{run}-{}",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::unique_params;

    #[test]
    fn test_usage() {
        assert_ne!(unique_params(), unique_params());
    }
}
