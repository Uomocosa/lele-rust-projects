use freenet_libp2p_bevy_plugin::net_id;

use crate::boxes;

/// Deterministic spawn x for a player, derived from their id so every client independently
/// computes the same position without coordinating.
#[must_use]
pub fn spawn_x_for_player(id: net_id::NetworkId) -> f32 {
    let bound = boxes::GROUND_WIDTH / 2.0 - boxes::BOX_SIZE / 2.0;
    let hashed = (*id).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let low24 = u32::try_from(hashed >> 40).unwrap_or(u32::MAX);
    let hi = u16::try_from(low24 >> 12).unwrap_or(u16::MAX);
    let lo = u16::try_from(low24 & 0xFFF).unwrap_or(u16::MAX);
    let frac = f32::from(hi).mul_add(4096.0, f32::from(lo)) / 16_777_216.0;
    frac.mul_add(2.0 * bound, -bound)
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
