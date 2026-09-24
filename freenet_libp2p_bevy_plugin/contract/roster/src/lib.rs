#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlayerId {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEntry {
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub updated_at: u64,
}

pub type RosterState = BTreeMap<PlayerId, PeerEntry>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub params: Vec<u8>,
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub updated_at: u64,
}

pub type DirectoryState = BTreeMap<String, DirectoryEntry>;

pub const DIRECTORY_LOBBY: &str = "__directory__";

fn is_directory(parameters: Parameters<'static>) -> bool {
    if let Ok((_, lobby)) =
        bincode::deserialize::<(String, String)>(parameters.as_ref())
    {
        return lobby == DIRECTORY_LOBBY;
    }
    bincode::deserialize::<(String, String, u128)>(parameters.as_ref())
        .is_ok_and(|(_, lobby, _)| lobby == DIRECTORY_LOBBY)
}

#[allow(dead_code)]
struct RosterContract;

fn merge_entry(existing: Option<PeerEntry>, incoming: PeerEntry) -> PeerEntry {
    match existing {
        Some(current) if current.updated_at >= incoming.updated_at => current,
        _ => incoming,
    }
}

fn merge_roster(mut base: RosterState, other: RosterState) -> RosterState {
    for (id, entry) in other {
        let merged = merge_entry(base.remove(&id), entry);
        base.insert(id, merged);
    }
    base
}

fn merge_directory_entry(
    existing: Option<DirectoryEntry>,
    incoming: DirectoryEntry,
) -> DirectoryEntry {
    match existing {
        Some(current) if current.updated_at >= incoming.updated_at => current,
        _ => incoming,
    }
}

fn merge_directory(mut base: DirectoryState, other: DirectoryState) -> DirectoryState {
    for (room, entry) in other {
        let merged = merge_directory_entry(base.remove(&room), entry);
        base.insert(room, merged);
    }
    base
}

fn decode_update(data: UpdateData<'static>) -> Option<Vec<u8>> {
    match data {
        UpdateData::State(state) => Some(state.as_ref().to_vec()),
        UpdateData::Delta(delta) => Some(delta.as_ref().to_vec()),
        UpdateData::StateAndDelta { state, .. } => Some(state.as_ref().to_vec()),
        _ => None,
    }
}

fn update_directory(
    state: State<'static>,
    data: Vec<UpdateData<'static>>,
) -> Result<UpdateModification<'static>, ContractError> {
    let mut merged: DirectoryState =
        bincode::deserialize(state.as_ref()).unwrap_or_default();
    for update in data {
        let Some(bytes) = decode_update(update) else {
            continue;
        };
        let incoming: DirectoryState =
            bincode::deserialize(&bytes).map_err(|e| ContractError::InvalidUpdateWithInfo {
                reason: e.to_string(),
            })?;
        merged = merge_directory(merged, incoming);
    }
    let new_state = State::from(bincode::serialize(&merged).unwrap());
    Ok(UpdateModification::valid(new_state))
}

#[contract]
impl ContractInterface for RosterContract {
    fn validate_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        if is_directory(parameters) {
            return bincode::deserialize::<DirectoryState>(state.as_ref())
                .map(|_| ValidateResult::Valid)
                .map_err(|_| ContractError::InvalidState);
        }
        bincode::deserialize::<RosterState>(state.as_ref())
            .map(|_| ValidateResult::Valid)
            .map_err(|_| ContractError::InvalidState)
    }

    fn update_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        if is_directory(parameters) {
            return update_directory(state, data);
        }
        let mut merged: RosterState = bincode::deserialize(state.as_ref()).unwrap_or_default();

        for update in data {
            let Some(bytes) = decode_update(update) else {
                continue;
            };
            let incoming: RosterState =
                bincode::deserialize(&bytes).map_err(|e| ContractError::InvalidUpdateWithInfo {
                    reason: e.to_string(),
                })?;
            merged = merge_roster(merged, incoming);
        }

        let new_state = State::from(bincode::serialize(&merged).unwrap());
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        if is_directory(parameters) {
            let directory = bincode::deserialize::<DirectoryState>(state.as_ref())
                .map_err(|_| ContractError::InvalidState)?;
            return Ok(StateSummary::from(bincode::serialize(&directory).unwrap()));
        }
        let roster = bincode::deserialize::<RosterState>(state.as_ref())
            .map_err(|_| ContractError::InvalidState)?;
        Ok(StateSummary::from(bincode::serialize(&roster).unwrap()))
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

    fn entry(peer_id: &str, updated_at: u64) -> PeerEntry {
        PeerEntry {
            peer_id: peer_id.to_string(),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{updated_at}")],
            updated_at,
        }
    }

    fn roster_state(bytes: RosterState) -> State<'static> {
        State::from(bincode::serialize(&bytes).unwrap())
    }

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let related = RelatedContracts::default();
        let empty = roster_state(RosterState::default());

        let result = RosterContract::validate_state(params.clone(), empty.clone(), related);
        assert!(matches!(result, Ok(ValidateResult::Valid)));

        let mut roster = RosterState::default();
        roster.insert(PlayerId { value: 1 }, entry("peer-1", 10));
        let update = vec![UpdateData::State(roster_state(roster.clone()))];

        let result = RosterContract::update_state(params.clone(), empty, update);
        assert!(result.is_ok());
        let new_state = result.unwrap().unwrap_valid();
        let decoded: RosterState = bincode::deserialize(new_state.as_ref()).unwrap();
        assert_eq!(decoded, roster);

        let result = RosterContract::summarize_state(params, roster_state(roster));
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_state_is_commutative() {
        let params = Parameters::from(Vec::new());
        let mut a = RosterState::default();
        a.insert(PlayerId { value: 1 }, entry("peer-1", 5));
        let mut b = RosterState::default();
        b.insert(PlayerId { value: 2 }, entry("peer-2", 7));

        let ab = RosterContract::update_state(
            params.clone(),
            roster_state(RosterState::default()),
            vec![
                UpdateData::State(roster_state(a.clone())),
                UpdateData::State(roster_state(b.clone())),
            ],
        )
        .unwrap()
        .unwrap_valid();

        let ba = RosterContract::update_state(
            params,
            roster_state(RosterState::default()),
            vec![
                UpdateData::State(roster_state(b)),
                UpdateData::State(roster_state(a)),
            ],
        )
        .unwrap()
        .unwrap_valid();

        assert_eq!(ab.as_ref(), ba.as_ref());
    }

    #[test]
    fn test_update_state_is_idempotent() {
        let params = Parameters::from(Vec::new());
        let mut roster = RosterState::default();
        roster.insert(PlayerId { value: 1 }, entry("peer-1", 5));

        let once = RosterContract::update_state(
            params.clone(),
            roster_state(RosterState::default()),
            vec![UpdateData::State(roster_state(roster.clone()))],
        )
        .unwrap()
        .unwrap_valid();

        let twice = RosterContract::update_state(
            params,
            once.clone(),
            vec![UpdateData::State(roster_state(roster))],
        )
        .unwrap()
        .unwrap_valid();

        assert_eq!(once.as_ref(), twice.as_ref());
    }

    #[test]
    fn test_update_state_is_associative() {
        let params = Parameters::from(Vec::new());
        let mut a = RosterState::default();
        a.insert(PlayerId { value: 1 }, entry("peer-1", 1));
        let mut b = RosterState::default();
        b.insert(PlayerId { value: 1 }, entry("peer-1-updated", 9));
        let mut c = RosterState::default();
        c.insert(PlayerId { value: 2 }, entry("peer-2", 3));

        let empty = roster_state(RosterState::default());

        let ab_then_c = RosterContract::update_state(
            params.clone(),
            empty.clone(),
            vec![
                UpdateData::State(roster_state(a.clone())),
                UpdateData::State(roster_state(b.clone())),
            ],
        )
        .unwrap()
        .unwrap_valid();
        let ab_then_c = RosterContract::update_state(
            params.clone(),
            ab_then_c,
            vec![UpdateData::State(roster_state(c.clone()))],
        )
        .unwrap()
        .unwrap_valid();

        let a_then_bc = RosterContract::update_state(
            params.clone(),
            empty,
            vec![UpdateData::State(roster_state(a))],
        )
        .unwrap()
        .unwrap_valid();
        let a_then_bc = RosterContract::update_state(
            params,
            a_then_bc,
            vec![
                UpdateData::State(roster_state(b)),
                UpdateData::State(roster_state(c)),
            ],
        )
        .unwrap()
        .unwrap_valid();

        assert_eq!(ab_then_c.as_ref(), a_then_bc.as_ref());
    }

    #[test]
    fn test_get_state_delta_carries_the_roster() {
        // Regression guard mirroring clicker_contract's history: get_state_delta must not
        // silently collapse to an empty delta, or peers catching up via delta stop syncing.
        let params = Parameters::from(Vec::new());
        let mut roster = RosterState::default();
        roster.insert(PlayerId { value: 1 }, entry("peer-1", 42));
        let state = roster_state(roster.clone());

        let summary = RosterContract::summarize_state(params.clone(), state.clone()).unwrap();
        let delta = RosterContract::get_state_delta(params, state, summary).unwrap();

        assert!(
            !delta.as_ref().is_empty(),
            "get_state_delta returned an empty delta"
        );
        let decoded: RosterState = bincode::deserialize(delta.as_ref()).unwrap();
        assert_eq!(decoded, roster);
    }

    #[test]
    fn test_invalid_state_rejected() {
        let bad_state = State::from(b"not a roster".to_vec());
        let result = RosterContract::validate_state(
            Parameters::from(Vec::new()),
            bad_state,
            RelatedContracts::default(),
        );
        assert!(
            matches!(result, Err(ContractError::InvalidState)),
            "expected InvalidState, got {result:?}"
        );
    }

    fn directory_params() -> Parameters<'static> {
        Parameters::from(
            bincode::serialize(&("ns".to_string(), DIRECTORY_LOBBY.to_string())).unwrap(),
        )
    }

    fn directory_entry(room: &str, updated_at: u64) -> DirectoryEntry {
        DirectoryEntry {
            params: vec![1, 2, 3],
            peer_id: format!("peer-{room}"),
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{updated_at}")],
            updated_at,
        }
    }

    fn directory_state(bytes: DirectoryState) -> State<'static> {
        State::from(bincode::serialize(&bytes).unwrap())
    }

    #[test]
    fn test_directory_branch_accepts_directory_state() {
        let params = directory_params();
        let related = RelatedContracts::default();
        let mut directory = DirectoryState::default();
        directory.insert("room-1".to_string(), directory_entry("room-1", 10));
        let result =
            RosterContract::validate_state(params.clone(), directory_state(directory), related);
        assert!(matches!(result, Ok(ValidateResult::Valid)));
    }

    #[test]
    fn test_directory_merge_is_lww_per_room() {
        let params = directory_params();
        let mut old = DirectoryState::default();
        old.insert("room-1".to_string(), directory_entry("room-1", 5));
        let mut fresh = DirectoryState::default();
        fresh.insert("room-1".to_string(), directory_entry("room-1", 9));
        fresh.insert("room-2".to_string(), directory_entry("room-2", 3));
        let merged = RosterContract::update_state(
            params,
            directory_state(old),
            vec![UpdateData::State(directory_state(fresh))],
        )
        .unwrap()
        .unwrap_valid();
        let decoded: DirectoryState = bincode::deserialize(merged.as_ref()).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded["room-1"].updated_at, 9);
        assert_eq!(decoded["room-2"].updated_at, 3);
    }

    #[test]
    fn test_directory_merge_is_commutative() {
        let params = directory_params();
        let mut a = DirectoryState::default();
        a.insert("room-1".to_string(), directory_entry("room-1", 5));
        let mut b = DirectoryState::default();
        b.insert("room-2".to_string(), directory_entry("room-2", 7));
        let ab = RosterContract::update_state(
            params.clone(),
            directory_state(DirectoryState::default()),
            vec![
                UpdateData::State(directory_state(a.clone())),
                UpdateData::State(directory_state(b.clone())),
            ],
        )
        .unwrap()
        .unwrap_valid();
        let ba = RosterContract::update_state(
            params,
            directory_state(DirectoryState::default()),
            vec![
                UpdateData::State(directory_state(b)),
                UpdateData::State(directory_state(a)),
            ],
        )
        .unwrap()
        .unwrap_valid();
        assert_eq!(ab.as_ref(), ba.as_ref());
    }
}
