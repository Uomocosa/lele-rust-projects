use std::collections::BTreeMap;

use crate::boxes;
use crate::roster;

pub type RosterState = BTreeMap<boxes::PlayerId, roster::PeerEntry>;

#[cfg(test)]
mod tests {
    use super::RosterState;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut state = RosterState::new();
        state.insert(
            [1; 32],
            roster::PeerEntry {
                peer_id: "peer-1".to_string(),
                addrs: vec![],
                seq: 1,
                signature: Vec::new(),
            },
        );
        assert_eq!(state.len(), 1);
    }
}
