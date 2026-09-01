use crate::engine;

pub fn spawn_x_for_player(id: engine::PlayerId) -> f32 {
    let bound = engine::GROUND_WIDTH / 2.0 - engine::BOX_SIZE / 2.0;
    let seed = u64::from_le_bytes([id[0], id[1], id[2], id[3], id[4], id[5], id[6], id[7]]);
    let hashed = seed.wrapping_mul(0x9E3779B97F4A7C15);
    let frac = (hashed >> 40) as f32 / (1u64 << 24) as f32;
    -bound + frac * (2.0 * bound)
}

#[cfg(test)]
mod tests {
    use super::spawn_x_for_player;
    use crate::engine;

    #[test]
    fn test_usage() {
        let bound = engine::GROUND_WIDTH / 2.0 - engine::BOX_SIZE / 2.0;
        let a = spawn_x_for_player([1; 32]);
        assert!((-bound..=bound).contains(&a));
        assert_eq!(a, spawn_x_for_player([1; 32]));
    }
}
