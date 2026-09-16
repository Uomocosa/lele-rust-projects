use serde::{Deserialize, Serialize};

use freenet_libp2p_bevy_plugin::net_id;

use crate::discovery;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CursorMsg {
    Click {
        owner: net_id::NetworkId,
        delta: i32,
    },
    Move {
        owner: net_id::NetworkId,
        pos: [f32; 2],
    },
    SyncReq {
        requester: net_id::NetworkId,
    },
    SyncAck {
        target: net_id::NetworkId,
        entries: Vec<(net_id::NetworkId, i32)>,
    },
    PexAsk {
        want_rooms: bool,
    },
    PexResp {
        peers: Vec<discovery::PeerHint>,
        rooms: Vec<(String, discovery::DirectoryEntry)>,
    },
    WantJoin {
        room: String,
    },
    Welcome {
        room: String,
        peers: Vec<String>,
        own_score: (net_id::NetworkId, i32),
        joining: Vec<String>,
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
        let req = CursorMsg::SyncReq {
            requester: net_id::NetworkId(7),
        };
        let encoded = bincode::serialize(&req).unwrap_or_default();
        let decoded: Option<CursorMsg> = bincode::deserialize(&encoded).ok();
        assert_eq!(decoded, Some(req));
        let ack = CursorMsg::SyncAck {
            target: net_id::NetworkId(7),
            entries: vec![(net_id::NetworkId(1), 3)],
        };
        let encoded = bincode::serialize(&ack).unwrap_or_default();
        let decoded: Option<CursorMsg> = bincode::deserialize(&encoded).ok();
        assert_eq!(decoded, Some(ack));
        let ask = CursorMsg::PexAsk { want_rooms: true };
        let encoded = bincode::serialize(&ask).unwrap_or_default();
        let decoded: Option<CursorMsg> = bincode::deserialize(&encoded).ok();
        assert_eq!(decoded, Some(ask));
        let want = CursorMsg::WantJoin {
            room: "alpha".to_string(),
        };
        let encoded = bincode::serialize(&want).unwrap_or_default();
        let decoded: Option<CursorMsg> = bincode::deserialize(&encoded).ok();
        assert_eq!(decoded, Some(want));
        let welcome = CursorMsg::Welcome {
            room: "alpha".to_string(),
            peers: vec!["peer-2".to_string()],
            own_score: (net_id::NetworkId(1), 3),
            joining: Vec::new(),
        };
        let encoded = bincode::serialize(&welcome).unwrap_or_default();
        let decoded: Option<CursorMsg> = bincode::deserialize(&encoded).ok();
        assert_eq!(decoded, Some(welcome));
    }
}
