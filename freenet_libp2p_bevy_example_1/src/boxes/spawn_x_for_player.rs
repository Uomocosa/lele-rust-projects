use crate::boxes;

/// Deterministic spawn x for a player, derived from their `PlayerId` so every client
/// (which cannot yet see the roster at `Startup`) independently computes the same
/// position for the same player without needing to coordinate.
pub fn spawn_x_for_player(id: boxes::PlayerId) -> f32 {
    let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;
    let hashed = id.0.wrapping_mul(0x9E3779B97F4A7C15);
    let frac = (hashed >> 40) as f32 / (1u64 << 24) as f32;
    -bound + frac * (2.0 * bound)
}

#[cfg(test)]
mod tests {
    use super::spawn_x_for_player;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;

        let a = spawn_x_for_player(boxes::PlayerId(1));
        let b = spawn_x_for_player(boxes::PlayerId(2));
        assert_ne!(a, b);
        assert!((-bound..=bound).contains(&a));
        assert!((-bound..=bound).contains(&b));

        assert_eq!(a, spawn_x_for_player(boxes::PlayerId(1)));
    }
}
