#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LobbyId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyMeta {
    pub lobby_id: LobbyId,
    pub max_members: u8,
    pub member_count: u8,
    pub owner: [u8; 32],
    pub created_at: u64,
    pub seq: u64,
}

pub type DirectoryState = BTreeMap<LobbyId, LobbyMeta>;

const MAX_LOBBIES: usize = 256;

fn is_valid_lobby_id(s: &str) -> bool {
    if s.is_empty() || s.len() > 32 {
        return false;
    }
    s.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn merge_entry(current: Option<LobbyMeta>, incoming: LobbyMeta) -> LobbyMeta {
    match current {
        Some(c) if c.seq >= incoming.seq => c,
        _ => incoming,
    }
}

#[allow(dead_code)]
struct DirectoryContract;

#[contract]
impl ContractInterface for DirectoryContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        let map: DirectoryState = bincode::deserialize(state.as_ref())
            .map_err(|_| ContractError::InvalidState)?;
        if map.len() > MAX_LOBBIES {
            return Err(ContractError::InvalidState);
        }
        for (id, meta) in &map {
            if !is_valid_lobby_id(&id.0) || !is_valid_lobby_id(&meta.lobby_id.0) {
                return Err(ContractError::InvalidState);
            }
            if meta.member_count > meta.max_members {
                return Err(ContractError::InvalidState);
            }
        }
        Ok(ValidateResult::Valid)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut base: DirectoryState =
            bincode::deserialize(state.as_ref()).unwrap_or_default();
        for update in data {
            let bytes = match update {
                UpdateData::State(s) => s.as_ref().to_vec(),
                UpdateData::Delta(d) => d.as_ref().to_vec(),
                UpdateData::StateAndDelta { state, .. } => state.as_ref().to_vec(),
                _ => continue,
            };
            let incoming: DirectoryState = match bincode::deserialize(&bytes) {
                Ok(v) => v,
                Err(e) => {
                    return Err(ContractError::InvalidUpdateWithInfo {
                        reason: e.to_string(),
                    })
                }
            };
            if incoming.len() > MAX_LOBBIES {
                continue;
            }
            for (id, meta) in incoming {
                if !is_valid_lobby_id(&id.0) {
                    continue;
                }
                if meta.member_count > meta.max_members {
                    continue;
                }
                let merged = merge_entry(base.remove(&id), meta);
                base.insert(id, merged);
            }
        }
        Ok(UpdateModification::valid(State::from(
            bincode::serialize(&base).unwrap(),
        )))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let map: DirectoryState = bincode::deserialize(state.as_ref())
            .map_err(|_| ContractError::InvalidState)?;
        Ok(StateSummary::from(bincode::serialize(&map).unwrap()))
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
    use super::*;

    fn state_of(map: DirectoryState) -> State<'static> {
        State::from(bincode::serialize(&map).unwrap())
    }

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let related = RelatedContracts::default();
        let empty = state_of(DirectoryState::default());
        assert!(matches!(
            DirectoryContract::validate_state(params, empty, related),
            Ok(ValidateResult::Valid)
        ));
    }
}
