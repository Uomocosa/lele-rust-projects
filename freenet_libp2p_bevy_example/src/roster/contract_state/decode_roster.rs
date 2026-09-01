use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::roster;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct HashedInput {
    tick: u64,
    hash: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct InputLogEntry {
    seq: u64,
    inputs: Vec<HashedInput>,
    signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Envelope {
    roster: roster::RosterState,
    input_log: BTreeMap<[u8; 32], InputLogEntry>,
}

pub fn decode_roster(bytes: &[u8]) -> Option<roster::RosterState> {
    if let Ok(envelope) = bincode::deserialize::<Envelope>(bytes) {
        return Some(envelope.roster);
    }
    bincode::deserialize::<roster::RosterState>(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::decode_roster;
    use crate::roster;

    #[test]
    fn test_usage() {
        let mut roster = roster::RosterState::default();
        roster.insert(
            [1; 32],
            roster::PeerEntry {
                peer_id: "peer".to_string(),
                addrs: vec![],
                seq: 1,
                signature: Vec::new(),
            },
        );

        let bare = bincode::serialize(&roster).unwrap();
        assert_eq!(decode_roster(&bare), Some(roster.clone()));

        let envelope = super::Envelope {
            roster: roster.clone(),
            input_log: Default::default(),
        };
        let envelope_bytes = bincode::serialize(&envelope).unwrap();
        assert_eq!(decode_roster(&envelope_bytes), Some(roster));

        assert!(decode_roster(b"junk").is_none());
    }
}
