use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::engine;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NetcodeMsg {
    /// Acknowledges receipt of a request. Resolves libp2p `request_response` pending-request
    /// state; a no-op for the game loop. Without replying to every inbound request the
    /// connection accumulates unanswered requests that degrade and stall.
    Ack,
    Commit {
        tick: u64,
        player_id: engine::PlayerId,
        hash: u64,
    },
    Reveal {
        tick: u64,
        player_id: engine::PlayerId,
        action: engine::Action,
    },
    StateHash {
        tick: u64,
        hash: u64,
    },
    /// Request an authoritative state snapshot from an established peer.
    /// A late joiner sends this on first connection.
    RequestSnapshot {
        player_id: engine::PlayerId,
    },
    /// Authoritative state snapshot reply carrying full EngineSimState (positions + velocities).
    Snapshot {
        tick: u64,
        bodies: BTreeMap<engine::PlayerId, (f32, f32, f32, f32)>,
        participants: Vec<engine::PlayerId>,
        from: engine::PlayerId,
        hash: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::NetcodeMsg;
    use crate::engine;

    #[test]
    fn test_usage() {
        let commit = NetcodeMsg::Commit {
            tick: 1,
            player_id: [3; 32],
            hash: 42,
        };
        let encoded = bincode::serialize(&commit);
        let decoded = encoded.ok().and_then(|e| bincode::deserialize(&e).ok());
        assert_eq!(decoded, Some(commit));

        let reveal = NetcodeMsg::Reveal {
            tick: 1,
            player_id: [3; 32],
            action: engine::Action::default(),
        };
        assert!(bincode::serialize(&reveal).is_ok());

        let req = NetcodeMsg::RequestSnapshot { player_id: [1; 32] };
        let encoded = bincode::serialize(&req).unwrap();
        let decoded: NetcodeMsg = bincode::deserialize(&encoded).unwrap();
        assert!(matches!(decoded, NetcodeMsg::RequestSnapshot { .. }));

        let mut bodies = std::collections::BTreeMap::new();
        bodies.insert([1; 32], (1.0, 2.0, 0.0, 0.0));
        let snapshot = NetcodeMsg::Snapshot {
            tick: 10,
            bodies,
            participants: vec![[1; 32], [2; 32]],
            from: [1; 32],
            hash: 42,
        };
        let encoded = bincode::serialize(&snapshot).unwrap();
        let decoded: NetcodeMsg = bincode::deserialize(&encoded).unwrap();
        assert_eq!(decoded, snapshot);
    }
}
