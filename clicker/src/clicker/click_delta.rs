use serde::{Deserialize, Serialize};

use freenet_libp2p_bevy_plugin::net_id;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClickDelta {
    pub owner: net_id::NetworkId,
    pub delta: i32,
}

#[cfg(test)]
mod tests {
    use super::ClickDelta;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let delta = ClickDelta {
            owner: net_id::NetworkId(7),
            delta: 1,
        };
        let encoded = bincode::serialize(&delta);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(delta));
    }
}
