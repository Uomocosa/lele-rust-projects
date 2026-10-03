use std::sync::atomic::{AtomicU32, Ordering};

pub static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn bump_counter() -> u32 {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}
