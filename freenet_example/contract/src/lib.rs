#![allow(unexpected_cfgs)]

use freenet_stdlib::prelude::*;

#[allow(dead_code)]
struct ClickerContract;

#[contract]
impl ContractInterface for ClickerContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        bincode::deserialize::<u64>(state.as_ref())
            .map(|_| ValidateResult::Valid)
            .map_err(|_| ContractError::InvalidState)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        _state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let new_count = data
            .into_iter()
            .next()
            .and_then(|d| match d {
                UpdateData::State(s) => Some(s),
                UpdateData::Delta(d) => Some(State::from(d.as_ref().to_vec())),
                _ => None,
            })
            .ok_or(ContractError::InvalidUpdate)?;
        let count = bincode::deserialize::<u64>(new_count.as_ref()).map_err(|e| {
            ContractError::InvalidUpdateWithInfo {
                reason: e.to_string(),
            }
        })?;
        let new_state = State::from(bincode::serialize(&count).unwrap());
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let count =
            bincode::deserialize::<u64>(state.as_ref()).map_err(|_| ContractError::InvalidState)?;
        let summary = StateSummary::from(bincode::serialize(&count).unwrap());
        Ok(summary)
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        _state: State<'static>,
        _summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        Ok(StateDelta::from(vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage() {
        let params = Parameters::from(Vec::new());
        let related = RelatedContracts::default();
        let state = State::from(bincode::serialize(&0u64).unwrap());

        let result = ClickerContract::validate_state(params.clone(), state.clone(), related);
        assert!(matches!(result, Ok(ValidateResult::Valid)));

        let update = vec![UpdateData::State(State::from(
            bincode::serialize(&42u64).unwrap(),
        ))];
        let result = ClickerContract::update_state(params.clone(), state, update);
        assert!(result.is_ok());
        let new_state = result.unwrap().unwrap_valid();
        let count: u64 = bincode::deserialize(new_state.as_ref()).unwrap();
        assert_eq!(count, 42);

        let result = ClickerContract::summarize_state(
            params,
            State::from(bincode::serialize(&42u64).unwrap()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_state_rejected() {
        let bad_state = State::from(b"short".to_vec());
        let result = ClickerContract::validate_state(
            Parameters::from(Vec::new()),
            bad_state,
            RelatedContracts::default(),
        );
        assert!(
            matches!(result, Err(ContractError::InvalidState)),
            "expected InvalidState, got {result:?}"
        );
    }
}
