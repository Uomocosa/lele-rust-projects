use serde::{Deserialize, Serialize};

use crate::engine;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    }
}
