use std::collections::BTreeMap;

use crate::net_id;
use crate::roster;

pub type RosterState = BTreeMap<net_id::NetworkId, roster::PeerEntry>;

#[cfg(test)]
mod tests {
    use super::RosterState;
    use crate::net_id;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut state = RosterState::new();
        state.insert(
            net_id::NetworkId(1),
            roster::PeerEntry {
                peer_id: "peer-1".to_string(),
                addrs: vec![],
                updated_at: 1,
            },
        );
        assert_eq!(state.len(), 1);
    }
}
