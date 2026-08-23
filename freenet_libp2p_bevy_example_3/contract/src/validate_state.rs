use crate::constants;
use crate::contract_state;
use crate::error;
use crate::params;
use crate::validate_roster;
use crate::verify_log_signature;

pub fn validate_state(
    state: &contract_state::ContractState,
    params: &params::Params,
) -> Result<(), error::Error> {
    validate_roster::validate_roster(&state.roster, params)?;
    for (key, log_entry) in state.input_log.iter() {
        if !state.roster.contains_key(key) {
            return Err(error::Error::IdentityNotInRoster);
        }
        if log_entry.inputs.len() > constants::MAX_INPUT_RING {
            return Err(error::Error::InputLogTooLarge);
        }
        verify_log_signature::verify_log_signature(key, log_entry)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey};

use crate::{
        contract_state, entry_bytes, error, hashed_input, input_log_entry, log_signed_bytes,
        params, peer_entry, roster_state,
    };

    use super::validate_state;

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

    #[test]
    fn test_usage() {
        let p = params::Params {
            namespace: [7; 32],
            max_members: 2,
        };
        let (member_key, member) = signed_member(1, 1);
        let (log_key, log) = signed_log(1, 1);

        let mut state = contract_state::ContractState::default();
        state.roster.insert(member_key, member);
        state.input_log.insert(log_key, log);
        assert!(validate_state(&state, &p).is_ok());

        let mut unsigned = state.clone();
        unsigned.input_log.get_mut(&log_key).unwrap().signature = Vec::new();
        assert_eq!(
            validate_state(&unsigned, &p),
            Err(error::Error::UnsignedInput)
        );
    }

    #[test]
    fn log_identity_must_be_in_roster() {
        let p = params::Params {
            namespace: [7; 32],
            max_members: 2,
        };
        let (log_key, log) = signed_log(1, 1);
        let mut state = contract_state::ContractState::default();
        state.input_log.insert(log_key, log);
        assert_eq!(
            validate_state(&state, &p),
            Err(error::Error::IdentityNotInRoster)
        );
        let _ = roster_state::RosterState::default();
    }
}
