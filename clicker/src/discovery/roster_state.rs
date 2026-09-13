use std::collections::BTreeMap;

use crate::discovery;

pub type RosterState = BTreeMap<discovery::PlayerId, discovery::PeerEntry>;

#[cfg(test)]
mod tests {
    use super::RosterState;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut roster = RosterState::new();
        roster.insert(
            discovery::PlayerId(1),
            discovery::PeerEntry {
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                updated_at: 1,
            },
        );
        assert_eq!(roster.len(), 1);
    }
}
