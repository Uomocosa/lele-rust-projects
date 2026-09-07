use serde::{Deserialize, Serialize};

use freenet_libp2p_bevy_plugin::net_id;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub entries: Vec<(net_id::NetworkId, i32)>,
    pub global: i32,
}

#[cfg(test)]
mod tests {
    use super::Snapshot;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let snapshot = Snapshot {
            entries: vec![(net_id::NetworkId(1), 3), (net_id::NetworkId(2), 5)],
            global: 8,
        };
        let bytes = clicker::encode_snapshot(&snapshot);
        let decoded = clicker::decode_snapshot(&bytes).unwrap();
        assert_eq!(decoded, snapshot);
        assert!(clicker::decode_snapshot(b"not a snapshot").is_none());
    }
}
