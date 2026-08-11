use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic counter yielding a unique `Parameters` payload per call, so every test joins its
/// own private roster contract instance and cannot be affected by other tests or network
/// participants.
pub fn unique_params() -> Vec<u8> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    format!("roster-test-{}", COUNTER.fetch_add(1, Ordering::Relaxed)).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::unique_params;

    #[test]
    fn test_usage() {
        assert_ne!(unique_params(), unique_params());
    }
}
