use std::time::Instant;

pub fn elapsed_nanos() -> u128 {
    Instant::now().elapsed().as_nanos()
}
