#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use freenet_stdlib::prelude::*;

#[allow(dead_code)]
struct GlobalCounterContract;

type Slots = BTreeMap<u64, u64>;

fn decode_slots(state: &[u8]) -> Result<Slots, ContractError> {
    bincode::deserialize::<Slots>(state).map_err(|_| ContractError::InvalidState)
}

fn encode_slots(slots: &Slots) -> State<'static> {
    State::from(bincode::serialize(slots).expect("serialize slots"))
}

#[contract]
impl ContractInterface for GlobalCounterContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        decode_slots(state.as_ref()).map(|_| ValidateResult::Valid)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut current = decode_slots(state.as_ref()).unwrap_or_default();
        for update in data {
            let bytes = match update {
                UpdateData::State(s) => Some(s.as_ref().to_vec()),
                UpdateData::Delta(d) => Some(d.as_ref().to_vec()),
                _ => None,
            };
            let Some(bytes) = bytes else {
                continue;
            };
            let incoming = decode_slots(&bytes).map_err(|_| ContractError::InvalidUpdate)?;
            for (tag, value) in incoming {
                let entry = current.entry(tag).or_insert(0);
                *entry = (*entry).max(value);
            }
        }
        let new_state = encode_slots(&current);
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let slots = decode_slots(state.as_ref())?;
        let summary =
            bincode::serialize(&slots).map_err(|e| ContractError::Deser(e.to_string()))?;
        Ok(StateSummary::from(summary))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        let ours = decode_slots(state.as_ref())?;
        let theirs: Slots = match bincode::deserialize(summary.as_ref()) {
            Ok(s) => s,
            Err(_) => return Ok(StateDelta::from(state.as_ref().to_vec())),
        };
        let mut delta = Slots::new();
        for (tag, value) in &ours {
            if theirs.get(tag).copied().unwrap_or(0) < *value {
                delta.insert(*tag, *value);
            }
        }
        Ok(StateDelta::from(bincode::serialize(&delta).map_err(
            |e| ContractError::Deser(e.to_string()),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slots(map: &[(u64, u64)]) -> State<'static> {
        let s: Slots = map.iter().copied().collect();
        encode_slots(&s)
    }

    fn params() -> Parameters<'static> {
        Parameters::from(Vec::new())
    }

    #[test]
    fn test_usage() {
        // Contract validity (CRDT laws, validate/summarize/delta) is verified by freenet_contract_harness::run_suite; this test only shows API wiring.
        let state = slots(&[(0, 5), (1, 3)]);
        let related = RelatedContracts::default();
        assert!(GlobalCounterContract::validate_state(params(), state.clone(), related).is_ok());

        let update = vec![UpdateData::State(slots(&[(1, 7)]))];
        let result = GlobalCounterContract::update_state(params(), state.clone(), update).unwrap();
        let next_state = result.unwrap_valid();

        let summary = GlobalCounterContract::summarize_state(params(), next_state.clone()).unwrap();
        let _delta = GlobalCounterContract::get_state_delta(params(), next_state, summary).unwrap();
    }
}
