use std::collections::BTreeMap;

use crate::contract_state;
use crate::error;
use crate::input_log_entry;
use crate::merge_input_log;
use crate::merge_roster;
use crate::roster_state;

pub fn merge_state(
    base: &mut contract_state::ContractState,
    incoming: contract_state::ContractState,
) -> Result<(), error::Error> {
    merge_roster::merge_roster(&mut base.roster, incoming.roster)?;
    merge_input_logs(&mut base.input_log, incoming.input_log, &base.roster)?;
    Ok(())
}

// needed helper:
fn merge_input_logs(
    base: &mut BTreeMap<[u8; 32], input_log_entry::InputLogEntry>,
    incoming: BTreeMap<[u8; 32], input_log_entry::InputLogEntry>,
    roster: &roster_state::RosterState,
) -> Result<(), error::Error> {
    for (key, entry) in incoming {
        if !roster.contains_key(&key) {
            return Err(error::Error::IdentityNotInRoster);
        }
        let existing = base.get(&key).cloned();
        let Some(merged) = merge_input_log::merge_input_log(existing.as_ref(), &key, &entry)?
        else {
            continue;
        };
        base.insert(key, merged);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ed25519_dalek::{Signer, SigningKey};

    use crate::{
        contract_state, entry_bytes, hashed_input, input_log_entry, log_signed_bytes, peer_entry,
    };

    use super::merge_state;

    fn signed_member(secret: u8, seq: u64) -> ([u8; 32], peer_entry::PeerEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let mut entry = peer_entry::PeerEntry {
            peer_id: format!("peer-{secret}"),
            addrs: Vec::new(),
            seq,
            signature: Vec::new(),
        };
        let sig = sk.sign(&entry_bytes::entry_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    fn signed_log(secret: u8, seq: u64) -> ([u8; 32], input_log_entry::InputLogEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let mut entry = input_log_entry::InputLogEntry {
            seq,
            inputs: vec![hashed_input::HashedInput {
                tick: seq,
                hash: seq,
            }],
            signature: Vec::new(),
        };
        let sig = sk.sign(&log_signed_bytes::log_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    fn state_with(single: bool) -> contract_state::ContractState {
        let (ka, member) = signed_member(1, 1);
        let (kb, member_b) = signed_member(2, 1);
        let mut roster = BTreeMap::new();
        roster.insert(ka, member);
        if !single {
            roster.insert(kb, member_b);
        }
        let (log_key, log) = signed_log(1, 1);
        let mut input_log = BTreeMap::new();
        input_log.insert(log_key, log);
        contract_state::ContractState { roster, input_log }
    }

    #[test]
    fn test_usage() {
        let a = state_with(true);
        let b = state_with(true);
        let mut base = contract_state::ContractState::default();
        merge_state(&mut base, a.clone()).unwrap();
        merge_state(&mut base, b.clone()).unwrap();
        assert_eq!(base.input_log.len(), 1);
        assert_eq!(base.roster.len(), 1);
    }
}
