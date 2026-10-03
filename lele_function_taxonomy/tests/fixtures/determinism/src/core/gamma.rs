use std::time::Instant;

pub fn gamma_elapsed() -> u128 {
    Instant::now().elapsed().as_nanos()
}
