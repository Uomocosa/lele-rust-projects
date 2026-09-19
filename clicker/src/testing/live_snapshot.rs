use freenet_libp2p_bevy_plugin::net_id;

use super::mesh::Mesh;
use crate::clicker;

#[must_use]
pub fn live_snapshot(mesh: &mut Mesh) -> Vec<u8> {
    let counts = mesh.counts();
    let first = counts.first().cloned().unwrap_or_default();
    let snapshot = clicker::Snapshot {
        entries: vec![
            (net_id::NetworkId(1), first.count(1)),
            (net_id::NetworkId(2), first.count(2)),
            (net_id::NetworkId(3), first.count(3)),
        ],
        global: first.global,
    };
    clicker::encode_snapshot(&snapshot)
}

#[cfg(test)]
mod tests {
    use super::live_snapshot;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut mesh = testing::Mesh::of(3);
        assert_ne!(live_snapshot(&mut mesh), Vec::<u8>::new());
    }
}
