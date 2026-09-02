#![allow(unexpected_cfgs)]

use std::collections::BTreeSet;

use freenet_stdlib::prelude::*;

#[allow(dead_code)]
struct SetContract;

fn decode_set(state: &[u8]) -> Result<BTreeSet<u64>, ContractError> {
    bincode::deserialize::<BTreeSet<u64>>(state).map_err(|_| ContractError::InvalidState)
}

fn encode_set(set: &BTreeSet<u64>) -> State<'static> {
    State::from(bincode::serialize(set).expect("serialize set"))
}

#[contract]
impl ContractInterface for SetContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        decode_set(state.as_ref()).map(|_| ValidateResult::Valid)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut current = decode_set(state.as_ref()).unwrap_or_default();
        for update in data {
            let bytes = match update {
                UpdateData::State(s) => Some(s.as_ref().to_vec()),
                UpdateData::Delta(d) => Some(d.as_ref().to_vec()),
                _ => None,
            };
            let Some(bytes) = bytes else { continue; };
            current.extend(decode_set(&bytes)?)
        }
        let new_state = encode_set(&current);
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let set = decode_set(state.as_ref())?;
        let summary = bincode::serialize(&set.len())
            .map_err(|e| ContractError::Deser(e.to_string()))?;
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
    use super::*;

    fn state(set: &[u64]) -> State<'static> {
        let s: BTreeSet<u64> = set.iter().copied().collect();
        encode_set(&s)
    }

    fn params() -> Parameters<'static> {
        Parameters::from(Vec::new())
    }

    #[test]
    fn test_usage() {
        // Contract validity (CRDT laws, validate/summarize/delta) is verified by freenet_contract_harness::run_suite; this test only shows API wiring.
        let st = state(&[1, 2, 3]);
        let related = RelatedContracts::default();
        assert!(SetContract::validate_state(params(), st.clone(), related).is_ok());

        let update = vec![UpdateData::State(state(&[4, 5]))];
        let result = SetContract::update_state(params(), st.clone(), update).unwrap();
        let next_state = result.unwrap_valid();

        let summary = SetContract::summarize_state(params(), next_state.clone()).unwrap();
        let _delta = SetContract::get_state_delta(params(), next_state, summary).unwrap();
    }
}