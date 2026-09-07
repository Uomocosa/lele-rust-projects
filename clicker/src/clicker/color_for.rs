use bevy::prelude::Color;

use freenet_libp2p_bevy_plugin::net_id;

#[must_use]
pub fn color_for(owner: net_id::NetworkId) -> Color {
    let digest = blake3::hash(&(*owner).to_le_bytes());
    let bytes = digest.as_bytes();
    let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
    let hue = f32::from(raw) % 360.0;
    Color::hsl(hue, 0.7, 0.5)
}

#[cfg(test)]
mod tests {
    use super::color_for;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let first = color_for(net_id::NetworkId(7));
        let again = color_for(net_id::NetworkId(7));
        assert_eq!(first, again);
        let other = color_for(net_id::NetworkId(8));
        assert_ne!(first, other);
    }
}
