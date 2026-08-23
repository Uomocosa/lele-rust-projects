use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Params {
    namespace: [u8; 32],
    max_members: u16,
}

/// Monotonic, run-unique bincode-serialized contract `Params` per call. Every test run deploys
/// to a fresh roster contract key instead of reusing a shared namespace, because roster
/// contracts persist on the network and stale entries from a prior run would pollute the next
/// run's assertions.
pub fn unique_params() -> Vec<u8> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let run = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut namespace = [0u8; 32];
    namespace[0..8].copy_from_slice(&run.to_le_bytes());
    namespace[8..16].copy_from_slice(&counter.to_le_bytes());
    bincode::serialize(&Params {
        namespace,
        max_members: 16,
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::unique_params;

    #[test]
    fn test_usage() {
        assert_ne!(unique_params(), unique_params());
    }
}
