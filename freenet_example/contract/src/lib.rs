use freenet_stdlib::prelude::*;

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
        state: State<'static>,
        _data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut count = bincode::deserialize::<u64>(state.as_ref())
            .map_err(|e| ContractError::InvalidUpdateWithInfo { reason: e.to_string() })?;
        count = count.wrapping_add(1);
        let new_state = State::from(bincode::serialize(&count).unwrap());
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let count = bincode::deserialize::<u64>(state.as_ref())
            .map_err(|_| ContractError::InvalidState)?;
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
