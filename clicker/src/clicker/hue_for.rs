use freenet_libp2p_bevy_plugin::net_id;

#[must_use]
pub fn hue_for(owner: net_id::NetworkId) -> f32 {
    let digest = blake3::hash(&(*owner).to_le_bytes());
    let bytes = digest.as_bytes();
    let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
    f32::from(raw) % 360.0
}

#[cfg(test)]
mod tests {
    use super::hue_for;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let first = hue_for(net_id::NetworkId(7));
        let again = hue_for(net_id::NetworkId(7));
        assert!((first - again).abs() < 0.001);
        assert!(first >= 0.0);
        assert!(first < 360.0);
    }
}
