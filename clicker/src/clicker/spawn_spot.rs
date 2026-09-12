use bevy::prelude::Vec2;
use core::f32::consts::TAU;

use freenet_libp2p_bevy_plugin::net_id;

const SLOTS: u64 = 8;
const RADIUS: f32 = 260.0;

#[must_use]
pub fn spawn_spot(own: net_id::NetworkId) -> Vec2 {
    let slot = (*own).checked_rem(SLOTS).unwrap_or(0);
    let slot8 = u8::try_from(slot).unwrap_or(0);
    let step = f32::from(slot8);
    let angle = step * TAU / 8.0;
    Vec2::new(RADIUS * angle.cos(), RADIUS * angle.sin())
}

#[cfg(test)]
mod tests {
    use super::spawn_spot;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let origin = spawn_spot(net_id::NetworkId(0));
        assert!((origin.length() - 260.0).abs() < 0.01);
        let other = spawn_spot(net_id::NetworkId(1));
        assert!((other.length() - 260.0).abs() < 0.01);
        assert!((other - origin).length() > 1.0);
        let wrap = spawn_spot(net_id::NetworkId(8));
        assert!((wrap - origin).length() < 0.01);
    }
}
