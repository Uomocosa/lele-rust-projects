#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct PlayerId(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    Square,
    Circle,
    Star,
    Triangle,
    Hex,
    Heart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tool {
    pub shape: Shape,
    pub color: [u8; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub tool: Tool,
    pub peer_id: String,
    pub addrs: Vec<String>,
    pub joined_at: u64,
    pub seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stamp {
    pub author: PlayerId,
    pub seq: u64,
    pub pos: [f32; 2],
    pub tool: Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoardState {
    pub members: BTreeMap<PlayerId, Member>,
    pub strokes: BTreeMap<(PlayerId, u64), Stamp>,
}

const MAX_MEMBERS: usize = 64;
const MAX_STROKES: usize = 4096;

pub fn tool_for(player: &PlayerId, lobby: &str) -> Tool {
    let bytes = bincode::serialize(&(player, lobby)).unwrap();
    let hash = blake3::hash(&bytes);
    let b = hash.as_bytes();
    let shapes = [
        Shape::Square,
        Shape::Circle,
        Shape::Star,
        Shape::Triangle,
        Shape::Hex,
        Shape::Heart,
    ];
    Tool {
        shape: shapes[(b[0] % 6) as usize],
        color: [b[1], b[2], b[3]],
    }
}

fn merge_member(current: Option<Member>, incoming: Member) -> Member {
    match current {
        Some(c) if c.seq >= incoming.seq => c,
        _ => incoming,
    }
}

#[allow(dead_code)]
struct BoardContract;

#[contract]
impl ContractInterface for BoardContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        let board: BoardState = bincode::deserialize(state.as_ref())
            .map_err(|_| ContractError::InvalidState)?;
        if board.members.len() > MAX_MEMBERS || board.strokes.len() > MAX_STROKES {
            return Err(ContractError::InvalidState);
        }
        Ok(ValidateResult::Valid)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let params_str = String::from_utf8_lossy(_parameters.as_ref()).to_string();
        let mut base: BoardState =
            bincode::deserialize(state.as_ref()).unwrap_or_default();
        for update in data {
            let bytes = match update {
                UpdateData::State(s) => s.as_ref().to_vec(),
                UpdateData::Delta(d) => d.as_ref().to_vec(),
                UpdateData::StateAndDelta { state, .. } => state.as_ref().to_vec(),
                _ => continue,
            };
            let incoming: BoardState = match bincode::deserialize(&bytes) {
                Ok(v) => v,
                Err(e) => {
                    return Err(ContractError::InvalidUpdateWithInfo {
                        reason: e.to_string(),
                    })
                }
            };
            for (pid, member) in incoming.members {
                if member.tool != tool_for(&pid, &params_str) {
                    continue;
                }
                let merged = merge_member(base.members.remove(&pid), member);
                base.members.insert(pid, merged);
            }
            if base.members.len() > MAX_MEMBERS {
                // keep lowest seq members deterministically
                let mut entries: Vec<_> = base.members.into_iter().collect();
                entries.sort_by_key(|(_, m)| m.seq);
                entries.truncate(MAX_MEMBERS);
                base.members = entries.into_iter().collect();
            }
            for ((author, seq), stamp) in incoming.strokes {
                if stamp.tool != tool_for(&author, &params_str) {
                    continue;
                }
                if !base.members.contains_key(&author) {
                    continue;
                }
                base.strokes.entry((author, seq)).or_insert(stamp);
            }
            while base.strokes.len() > MAX_STROKES {
                if let Some(k) = base.strokes.keys().next().cloned() {
                    base.strokes.remove(&k);
                } else {
                    break;
                }
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
        let board: BoardState = bincode::deserialize(state.as_ref())
            .map_err(|_| ContractError::InvalidState)?;
        Ok(StateSummary::from(bincode::serialize(&board).unwrap()))
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

    fn state_of(s: BoardState) -> State<'static> {
        State::from(bincode::serialize(&s).unwrap())
    }

    #[test]
    fn test_usage() {
        let params = Parameters::from(b"lobby-a".to_vec());
        let related = RelatedContracts::default();
        let empty = state_of(BoardState::default());
        assert!(matches!(
            BoardContract::validate_state(params, empty, related),
            Ok(ValidateResult::Valid)
        ));
    }

    #[test]
    fn test_tool_for_deterministic() {
        let pid = PlayerId([1; 32]);
        let a = tool_for(&pid, "lobby-x");
        let b = tool_for(&pid, "lobby-x");
        assert_eq!(a, b);
        let c = tool_for(&pid, "lobby-y");
        assert_ne!(a.color, c.color);
    }
}
