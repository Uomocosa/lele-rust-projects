use std::sync::atomic::{AtomicU32, Ordering};

pub static TALLY: AtomicU32 = AtomicU32::new(0);

pub fn runs_hidden_closure() -> u32 {
    let read = || TALLY.load(Ordering::SeqCst);
    read()
}
