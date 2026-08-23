#![allow(unexpected_cfgs)]

use freenet_stdlib::prelude::*;

pub mod constants;
pub mod decode_params;
pub mod decode_update;
pub mod entry_bytes;
pub mod error;
pub mod merge_entry;
pub mod merge_roster;
pub mod params;
pub mod peer_entry;
pub mod roster_state;
pub mod validate_entry;
pub mod validate_roster;
pub mod verify_entry_signature;

pub use constants::MAX_ADDRS;
pub use error::Error;
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
        let roster: RosterState =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        validate_roster::validate_roster(&roster, &params)
            .map_err(|_| ContractError::InvalidState)?;
        Ok(ValidateResult::Valid)
    }

    fn update_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let params = decode_params::decode_params(parameters.as_ref())
            .map_err(|e| ContractError::Deser(e.to_string()))?;
        let mut roster: RosterState = bincode::deserialize(state.as_ref()).unwrap_or_default();
        let guard = validate_roster::validate_roster(&roster, &params);
        if let Err(e) = guard {
            return Err(ContractError::InvalidUpdateWithInfo {
                reason: e.to_string(),
            });
        }
        for update in data {
            let Some(bytes) = decode_update::decode_update(update) else {
                continue;
            };
            let incoming: RosterState =
                bincode::deserialize(&bytes).map_err(|e| ContractError::InvalidUpdateWithInfo {
                    reason: e.to_string(),
                })?;
            merge_roster::merge_roster(&mut roster, incoming).map_err(|e| {
                ContractError::InvalidUpdateWithInfo {
                    reason: e.to_string(),
                }
            })?;
        }
        let new_state = State::from(
            bincode::serialize(&roster).map_err(|e| ContractError::Deser(e.to_string()))?,
        );
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let roster: RosterState =
            bincode::deserialize(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        let summary =
            bincode::serialize(&roster).map_err(|e| ContractError::Deser(e.to_string()))?;
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

    fn roster_state(bytes: &RosterState) -> State<'static> {
        State::from(bincode::serialize(bytes).unwrap())
    }

    fn signed(secret: u8, seq: u64) -> ([u8; 32], PeerEntry) {
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

    fn one(key: [u8; 32], entry: PeerEntry) -> RosterState {
        let mut map = BTreeMap::new();
        map.insert(key, entry);
        map
    }

    fn apply_roster(base: &RosterState, updates: &[RosterState]) -> RosterState {
        let mut current = base.clone();
        for upd in updates {
            let data = vec![UpdateData::State(State::from(
                bincode::serialize(upd).unwrap(),
            ))];
            let new = MembershipContract::update_state(params(), roster_state(&current), data)
                .unwrap()
                .unwrap_valid();
            current = bincode::deserialize(new.as_ref()).unwrap();
        }
        current
    }

    fn update_err(base: &RosterState, upd: &RosterState) -> ContractError {
        let data = vec![UpdateData::State(State::from(
            bincode::serialize(upd).unwrap(),
        ))];
        MembershipContract::update_state(params(), roster_state(base), data).unwrap_err()
    }

    #[test]
    fn test_usage() {
        let (key, entry) = signed(1, 1);
        let mut roster = RosterState::new();
        roster.insert(key, entry);
        let result = MembershipContract::validate_state(
            params(),
            roster_state(&roster),
            RelatedContracts::default(),
        );
        assert!(matches!(result, Ok(ValidateResult::Valid)));
    }

    #[test]
    fn test_merge_is_commutative() {
        let (ka, ea) = signed(1, 1);
        let (kb, eb) = signed(2, 1);
        let a = one(ka, ea);
        let b = one(kb, eb);

        let ab = apply_roster(&RosterState::new(), &[a.clone(), b.clone()]);
        let ba = apply_roster(&RosterState::new(), &[b, a]);
        assert_eq!(ab, ba);
        assert_eq!(ab.len(), 2);
    }

    #[test]
    fn test_merge_is_idempotent() {
        let (ka, ea) = signed(1, 1);
        let a = one(ka, ea);

        let once = apply_roster(&RosterState::new(), std::slice::from_ref(&a));
        let twice = apply_roster(&RosterState::new(), &[a.clone(), a]);
        assert_eq!(once, twice);
    }

    #[test]
    fn test_merge_is_associative() {
        let (ka, ea) = signed(1, 1);
        let (kb, eb) = signed(2, 1);
        let (kc, ec) = signed(3, 1);
        let a = one(ka, ea);
        let b = one(kb, eb);
        let c = one(kc, ec);

        let ab_then_c = apply_roster(&RosterState::new(), &[a.clone(), b.clone(), c.clone()]);
        let a_then_bc = apply_roster(&RosterState::new(), &[a, b, c]);
        assert_eq!(ab_then_c, a_then_bc);
    }

    #[test]
    fn test_invalid_state_rejected() {
        let bad = State::from(b"not a roster".to_vec());
        let result = MembershipContract::validate_state(params(), bad, RelatedContracts::default());
        assert!(matches!(result, Err(ContractError::InvalidState)));
    }

    #[test]
    fn test_rewind_rejected() {
        let (ka, ea) = signed(1, 2);
        let (_, eb) = signed(1, 1);
        let base = one(ka, ea);
        let rewind = one(ka, eb);
        let err = update_err(&base, &rewind);
        assert!(err.to_string().contains("rewind"));
    }

    #[test]
    fn test_wrong_signer_rejected() {
        let (ka, ea) = signed(1, 2);
        let (_, stolen) = signed(9, 3);
        let base = one(ka, ea);
        let attack = one(ka, stolen);
        let err = update_err(&base, &attack);
        assert!(err.to_string().contains("signature"));
    }

    #[test]
    fn test_unsigned_new_member_rejected() {
        let (key, mut entry) = signed(3, 1);
        entry.signature = vec![];
        let unsigned = one(key, entry);
        let err = update_err(&RosterState::new(), &unsigned);
        assert!(err.to_string().contains("signature"));
    }
}
