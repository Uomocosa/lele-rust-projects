#![allow(unexpected_cfgs)]

use freenet_stdlib::prelude::*;

pub mod constants;
pub mod contract_state;
pub mod decode_params;
pub mod decode_state;
pub mod decode_update;
pub mod entry_bytes;
pub mod error;
pub mod hashed_input;
pub mod input_log_entry;
pub mod log_signed_bytes;
pub mod merge_entry;
pub mod merge_input_log;
pub mod merge_roster;
pub mod merge_state;
pub mod params;
pub mod peer_entry;
pub mod roster_state;
pub mod validate_entry;
pub mod validate_roster;
pub mod validate_state;
pub mod verify_entry_signature;
pub mod verify_log_signature;

pub use constants::{MAX_ADDRS, MAX_INPUT_RING};
pub use contract_state::ContractState;
pub use error::Error;
pub use hashed_input::HashedInput;
pub use input_log_entry::InputLogEntry;
pub use params::Params;
pub use peer_entry::PeerEntry;
pub use roster_state::RosterState;

struct MembershipContract;

#[contract]
impl ContractInterface for MembershipContract {
    fn validate_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        let params = decode_params::decode_params(parameters.as_ref())
            .map_err(|e| ContractError::Deser(e.to_string()))?;
        let cs = decode_state::decode_state(state.as_ref()).ok_or(ContractError::InvalidState)?;
        validate_state::validate_state(&cs, &params).map_err(|_| ContractError::InvalidState)?;
        Ok(ValidateResult::Valid)
    }

    fn update_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let params = decode_params::decode_params(parameters.as_ref())
            .map_err(|e| ContractError::Deser(e.to_string()))?;
        let mut cs = decode_state::decode_state(state.as_ref()).unwrap_or_default();
        let guard = validate_state::validate_state(&cs, &params);
        if let Err(e) = guard {
            return Err(ContractError::InvalidUpdateWithInfo {
                reason: e.to_string(),
            });
        }
        for update in data {
            let Some(bytes) = decode_update::decode_update(update) else {
                continue;
            };
            let incoming = decode_state::decode_state(&bytes).ok_or_else(|| {
                ContractError::InvalidUpdateWithInfo {
                    reason: "cannot decode incoming update state".to_string(),
                }
            })?;
            validate_state::validate_state(&incoming, &params).map_err(|e| {
                ContractError::InvalidUpdateWithInfo {
                    reason: e.to_string(),
                }
            })?;
            merge_state::merge_state(&mut cs, incoming).map_err(|e| {
                ContractError::InvalidUpdateWithInfo {
                    reason: e.to_string(),
                }
            })?;
        }
        let new_state =
            State::from(bincode::serialize(&cs).map_err(|e| ContractError::Deser(e.to_string()))?);
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let cs = decode_state::decode_state(state.as_ref()).ok_or(ContractError::InvalidState)?;
        let summary = bincode::serialize(&cs).map_err(|e| ContractError::Deser(e.to_string()))?;
        Ok(StateSummary::from(summary))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        Ok(StateDelta::from(state.as_ref().to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ed25519_dalek::{Signer, SigningKey};

    use super::*;

    fn params() -> Parameters<'static> {
        Parameters::from(
            bincode::serialize(&Params {
                namespace: [7; 32],
                max_members: 16,
            })
            .unwrap(),
        )
    }

    fn state_bytes(state: &ContractState) -> State<'static> {
        State::from(bincode::serialize(state).unwrap())
    }

    fn roster_entry(secret: u8, seq: u64) -> ([u8; 32], PeerEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let mut entry = PeerEntry {
            peer_id: format!("peer-{secret}"),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{seq}")],
            seq,
            signature: Vec::new(),
        };
        let sig = sk.sign(&entry_bytes::entry_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    fn log_entry(secret: u8, seq: u64, count: usize) -> ([u8; 32], InputLogEntry) {
        let sk = SigningKey::from_bytes(&[secret; 32]);
        let vk = sk.verifying_key();
        let inputs = (0..count)
            .map(|i| HashedInput {
                tick: seq + i as u64,
                hash: seq + i as u64,
            })
            .collect::<Vec<_>>();
        let mut entry = InputLogEntry {
            seq,
            inputs,
            signature: Vec::new(),
        };
        let sig = sk.sign(&log_signed_bytes::log_signed_bytes(&entry));
        entry.signature = sig.to_bytes().to_vec();
        (vk.to_bytes(), entry)
    }

    fn one(secret: u8, seq: u64) -> ContractState {
        let (key, entry) = roster_entry(secret, seq);
        let (log_key, log) = log_entry(secret, seq, 2);
        let mut roster = BTreeMap::new();
        roster.insert(key, entry);
        let mut input_log = BTreeMap::new();
        input_log.insert(log_key, log);
        ContractState { roster, input_log }
    }

    fn apply(base: &ContractState, updates: &[ContractState]) -> ContractState {
        let mut current = base.clone();
        for upd in updates {
            let data = vec![UpdateData::State(State::from(
                bincode::serialize(upd).unwrap(),
            ))];
            let new = MembershipContract::update_state(params(), state_bytes(&current), data)
                .unwrap()
                .unwrap_valid();
            current = bincode::deserialize(new.as_ref()).unwrap();
        }
        current
    }

    fn update_err(base: &ContractState, upd: &ContractState) -> ContractError {
        let data = vec![UpdateData::State(State::from(
            bincode::serialize(upd).unwrap(),
        ))];
        MembershipContract::update_state(params(), state_bytes(base), data).unwrap_err()
    }

    #[test]
    fn test_usage() {
        let state = one(1, 1);
        let result = MembershipContract::validate_state(
            params(),
            state_bytes(&state),
            RelatedContracts::default(),
        );
        assert!(matches!(result, Ok(ValidateResult::Valid)));
    }

    #[test]
    fn test_merge_is_commutative() {
        let a = one(1, 1);
        let b = one(2, 1);

        let ab = apply(&ContractState::default(), &[a.clone(), b.clone()]);
        let ba = apply(&ContractState::default(), &[b, a]);
        assert_eq!(ab, ba);
        assert_eq!(ab.roster.len(), 2);
        assert_eq!(ab.input_log.len(), 2);
    }

    #[test]
    fn test_merge_is_idempotent() {
        let a = one(1, 1);

        let once = apply(&ContractState::default(), std::slice::from_ref(&a));
        let twice = apply(&ContractState::default(), &[a.clone(), a]);
        assert_eq!(once, twice);
    }

    #[test]
    fn test_merge_is_associative() {
        let a = one(1, 1);
        let b = one(2, 1);
        let c = one(3, 1);

        let ab_then_c = apply(
            &ContractState::default(),
            &[a.clone(), b.clone(), c.clone()],
        );
        let a_then_bc = apply(&ContractState::default(), &[a, b, c]);
        assert_eq!(ab_then_c, a_then_bc);
    }

    #[test]
    fn test_unsigned_input_rejected() {
        let (key, entry) = roster_entry(1, 1);
        let (log_key, mut log) = log_entry(1, 1, 2);
        log.signature = Vec::new();
        let mut roster = BTreeMap::new();
        roster.insert(key, entry);
        let mut input_log = BTreeMap::new();
        input_log.insert(log_key, log);
        let unsigned = ContractState { roster, input_log };
        let err = update_err(&ContractState::default(), &unsigned);
        assert!(err.to_string().contains("not signed"));
    }

    #[test]
    fn test_log_rewind_rejected() {
        let base = one(1, 2);
        let (log_key, rewind_log) = log_entry(1, 1, 2);
        let (key, entry) = roster_entry(1, 2);
        let mut roster = BTreeMap::new();
        roster.insert(key, entry);
        let mut input_log = BTreeMap::new();
        input_log.insert(log_key, rewind_log);
        let rewind = ContractState { roster, input_log };
        let err = update_err(&base, &rewind);
        assert!(err.to_string().contains("log rewinds"));
    }

    #[test]
    fn test_over_cap_input_rejected() {
        let (key, entry) = roster_entry(1, 1);
        let (log_key, log) = log_entry(1, 1, MAX_INPUT_RING + 1);
        let mut roster = BTreeMap::new();
        roster.insert(key, entry);
        let mut input_log = BTreeMap::new();
        input_log.insert(log_key, log);
        let over = ContractState { roster, input_log };
        let err = update_err(&ContractState::default(), &over);
        assert!(err.to_string().contains("ring cap"));
    }
}
