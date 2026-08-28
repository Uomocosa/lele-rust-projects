#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use freenet_stdlib::prelude::*;

#[allow(dead_code)]
struct ClickerContract;

type Slots = BTreeMap<u64, u64>;

fn decode_slots(state: &[u8]) -> Result<Slots, ContractError> {
    bincode::deserialize::<Slots>(state).map_err(|_| ContractError::InvalidState)
}

fn encode_slots(slots: &Slots) -> State<'static> {
    State::from(bincode::serialize(slots).expect("serialize slots"))
}

#[contract]
impl ContractInterface for ClickerContract {
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

    fn dbg_total(bytes: &[u8]) -> u64 {
        decode_slots(bytes)
            .map(|s| s.values().sum())
            .unwrap_or(0)
    }

    #[test]
    fn test_usage() {
        let related = RelatedContracts::default();
        assert!(
            ClickerContract::validate_state(params(), slots(&[(0, 5), (1, 3)]), related).is_ok()
        );

        let update = vec![UpdateData::State(slots(&[(1, 7)]))];
        let result =
            ClickerContract::update_state(params(), slots(&[(0, 5), (1, 3)]), update).unwrap();
        let merged = dbg_total(result.unwrap_valid().as_ref());
        assert_eq!(merged, 12);

        let summary = ClickerContract::summarize_state(params(), slots(&[(0, 5), (1, 3)])).unwrap();
        let summary_slots: Slots = bincode::deserialize(summary.as_ref()).unwrap();
        assert_eq!(summary_slots.get(&0), Some(&5));
        assert_eq!(summary_slots.get(&1), Some(&3));

        let delta = ClickerContract::get_state_delta(
            params(),
            slots(&[(0, 5), (1, 3)]),
            StateSummary::from(bincode::serialize(&Slots::from([(0u64, 5u64)])).unwrap()),
        )
        .unwrap();
        let got: Slots = bincode::deserialize(delta.as_ref()).unwrap();
        assert_eq!(got.get(&0), None);
        assert_eq!(got.get(&1), Some(&3));

        let garbage = ClickerContract::get_state_delta(
            params(),
            slots(&[(0, 5)]),
            StateSummary::from(b"not-bincode".to_vec()),
        )
        .unwrap();
        assert_eq!(dbg_total(garbage.as_ref()), 5);
    }

    #[test]
    fn test_equal_totals_masked_divergence_detected() {
        let summary_a =
            ClickerContract::summarize_state(params(), slots(&[(0, 4), (1, 4)])).unwrap();
        let summary_b =
            ClickerContract::summarize_state(params(), slots(&[(0, 8)])).unwrap();
        assert_ne!(summary_a.as_ref(), summary_b.as_ref());
    }

    #[test]
    fn test_merge_is_idempotent_max() {
        let update = vec![UpdateData::State(slots(&[(0, 9)]))];
        let r1 = ClickerContract::update_state(params(), slots(&[(0, 5)]), update.clone()).unwrap();
        let r2 = ClickerContract::update_state(params(), slots(&[(0, 5)]), update).unwrap();
        let s1 = r1.unwrap_valid();
        let s2 = r2.unwrap_valid();
        assert_eq!(s1.as_ref(), s2.as_ref());
        assert_eq!(dbg_total(s2.as_ref()), 9);
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
