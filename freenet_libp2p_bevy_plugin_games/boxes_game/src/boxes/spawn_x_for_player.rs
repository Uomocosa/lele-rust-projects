use freenet_libp2p_bevy_plugin::net_id;

use crate::boxes;

/// Deterministic spawn x for a player, derived from their id so every client independently
/// computes the same position without coordinating.
pub fn spawn_x_for_player(id: net_id::NetworkId) -> f32 {
    let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;
    let hashed = (*id).wrapping_mul(0x9E3779B97F4A7C15);
    let frac = (hashed >> 40) as f32 / (1u64 << 24) as f32;
    -bound + frac * (2.0 * bound)
}

#[cfg(test)]
mod tests {
    use super::spawn_x_for_player;
    use crate::boxes;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;

        let a = spawn_x_for_player(net_id::NetworkId(1));
        let b = spawn_x_for_player(net_id::NetworkId(2));
        assert_ne!(a, b);
        assert!((-bound..=bound).contains(&a));
        assert!((-bound..=bound).contains(&b));

        assert_eq!(a, spawn_x_for_player(net_id::NetworkId(1)));
    }
}
