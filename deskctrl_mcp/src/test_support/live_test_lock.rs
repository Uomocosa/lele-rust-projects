use std::sync::OnceLock;

use tokio::sync::Mutex;

/// Serializes live tests against each other (they share the X display / Telegram chat and can
/// race — e.g. a window closing mid-list from a concurrently running test) without forcing
/// `--test-threads=1` on the whole suite. `tokio::sync::Mutex` rather than `std::sync::Mutex`
/// since async live tests hold the guard across `.await` (clippy's `await_holding_lock` is real:
/// a std guard held there can stall the executor thread other tasks are on).
pub fn live_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

// no test_usage necessary
