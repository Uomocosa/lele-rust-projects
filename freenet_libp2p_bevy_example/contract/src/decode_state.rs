use std::collections::BTreeMap;

use crate::contract_state;
use crate::roster_state;

pub fn decode_state(bytes: &[u8]) -> Option<contract_state::ContractState> {
    if let Ok(cs) = bincode::deserialize::<contract_state::ContractState>(bytes) {
        return Some(cs);
    }
    if let Ok(roster) = bincode::deserialize::<roster_state::RosterState>(bytes) {
        return Some(contract_state::ContractState {
            roster,
            input_log: BTreeMap::new(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::{contract_state, peer_entry, roster_state};

    use super::decode_state;

    fn bare_roster() -> roster_state::RosterState {
        let mut map = roster_state::RosterState::new();
        map.insert(
            [1; 32],
            peer_entry::PeerEntry {
                peer_id: "peer".to_string(),
                addrs: Vec::new(),
                seq: 1,
                signature: Vec::new(),
            },
        );
        map
    }

    #[test]
    fn test_usage() {
        let roster = bare_roster();
        let bare_bytes = bincode::serialize(&roster).unwrap();
        let decoded = decode_state(&bare_bytes).unwrap();
        assert_eq!(decoded.roster, roster);
        assert!(decoded.input_log.is_empty());

        let envelope = contract_state::ContractState {
            roster,
            input_log: Default::default(),
        };
        let envelope_bytes = bincode::serialize(&envelope).unwrap();
        assert_eq!(decode_state(&envelope_bytes), Some(envelope));

        assert!(decode_state(b"not a roster").is_none());
    }
}
