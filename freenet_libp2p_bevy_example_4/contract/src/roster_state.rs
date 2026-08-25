use std::collections::BTreeMap;

use crate::peer_entry;

pub type RosterState = BTreeMap<[u8; 32], peer_entry::PeerEntry>;

#[cfg(test)]
mod tests {
    use crate::peer_entry;

    use super::RosterState;

    #[test]
    fn test_usage() {
        let mut roster = RosterState::new();
        roster.insert(
            [7; 32],
            peer_entry::PeerEntry {
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                seq: 1,
                signature: Vec::new(),
            },
        );
        assert_eq!(roster.len(), 1);
        assert!(roster.contains_key(&[7; 32]));
    }
}
