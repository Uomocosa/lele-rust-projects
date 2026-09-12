use serde::{Deserialize, Serialize};

use freenet_libp2p_bevy_plugin::net_id;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CursorMsg {
    Click {
        owner: net_id::NetworkId,
        delta: i32,
    },
    Move {
        owner: net_id::NetworkId,
        pos: [f32; 2],
    },
}

#[cfg(test)]
mod tests {
    use super::CursorMsg;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let click = CursorMsg::Click {
            owner: net_id::NetworkId(7),
            delta: 1,
        };
        let encoded = bincode::serialize(&click);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(click));
        let mv = CursorMsg::Move {
            owner: net_id::NetworkId(7),
            pos: [12.5, -3.25],
        };
        let encoded = bincode::serialize(&mv);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(mv));
    }
}
