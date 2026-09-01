use freenet_stdlib::prelude::*;

use crate::suite_config;

pub fn run_suite<C: ContractInterface>(cfg: suite_config::SuiteConfig) {
    assert_validate_accepts_gen::<C>(&cfg);
    assert_validate_rejects_garbage::<C>(&cfg);
    assert_summarize_deterministic::<C>(&cfg);
    assert_summarize_detects_structural_divergence::<C>(&cfg);
    assert_delta_nonempty_and_roundtrips::<C>(&cfg);
    assert_delta_handles_bad_summary::<C>(&cfg);
    assert_update_idempotent::<C>(&cfg);
    assert_update_commutative::<C>(&cfg);
    assert_update_associative::<C>(&cfg);
    assert_update_reads_data_not_state_plus1::<C>(&cfg);
    assert_update_empty_and_unknown_noop::<C>(&cfg);
    assert_update_rejects_garbage_data_without_panic::<C>(&cfg);
}

// needed helper:
fn assert_validate_accepts_gen<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let result = C::validate_state(cfg.params.clone(), state, RelatedContracts::default());
    assert!(
        result.is_ok(),
        "validate_accepts_gen: gen_state should be valid"
    );
}

// needed helper:
fn assert_validate_rejects_garbage<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    for bad in [Vec::new(), b"not-bincode".to_vec(), vec![0xFF; 32]] {
        let result = C::validate_state(
            cfg.params.clone(),
            State::from(bad),
            RelatedContracts::default(),
        );
        assert!(
            result.is_err(),
            "validate_rejects_garbage: garbage state should be rejected"
        );
    }
}

// needed helper:
fn assert_summarize_deterministic<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let Ok(a) = C::summarize_state(cfg.params.clone(), state.clone()) else {
        assert!(false, "summarize_deterministic: first summarize failed");
        return;
    };
    let Ok(b) = C::summarize_state(cfg.params.clone(), state) else {
        assert!(false, "summarize_deterministic: second summarize failed");
        return;
    };
    assert_eq!(
        a.as_ref(),
        b.as_ref(),
        "summarize_deterministic: same state must give same summary"
    );
}

// needed helper:
fn assert_summarize_detects_structural_divergence<C: ContractInterface>(
    cfg: &suite_config::SuiteConfig,
) {
    let Some(divergent) = cfg.gen_divergent_equal_total else {
        return;
    };
    let Some((a, b)) = divergent() else {
        return;
    };
    let Ok(sa) = C::summarize_state(cfg.params.clone(), a) else {
        assert!(false, "summarize divergent a failed");
        return;
    };
    let Ok(sb) = C::summarize_state(cfg.params.clone(), b) else {
        assert!(false, "summarize divergent b failed");
        return;
    };
    assert_ne!(
        sa.as_ref(),
        sb.as_ref(),
        "summarize_detects_structural_divergence: equal totals with different shape must have different summaries (per-tag summary required)"
    );
}

// needed helper:
fn assert_delta_nonempty_and_roundtrips<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let base = (cfg.gen_state)();
    let update = (cfg.gen_update)(&base);
    let Some(ahead) = try_apply::<C>(cfg.params.clone(), base.clone(), vec![update]) else {
        assert!(false, "delta_nonempty_and_roundtrips: apply ahead failed");
        return;
    };
    if ahead.as_ref() == base.as_ref() {
        return;
    }
    let Ok(summary) = C::summarize_state(cfg.params.clone(), base.clone()) else {
        assert!(false, "summarize base for delta failed");
        return;
    };
    let Ok(delta) = C::get_state_delta(cfg.params.clone(), ahead.clone(), summary) else {
        assert!(false, "get_state_delta failed");
        return;
    };
    assert!(
        !delta.as_ref().is_empty(),
        "delta_nonempty_and_roundtrips: empty delta disables anti-entropy"
    );
    let Ok(merged) = C::update_state(cfg.params.clone(), base, vec![UpdateData::Delta(delta)])
    else {
        assert!(false, "delta update failed");
        return;
    };
    let Some(merged_state) = merged.new_state else {
        assert!(false, "delta merge returned no state");
        return;
    };
    assert_eq!(
        merged_state.as_ref(),
        ahead.as_ref(),
        "delta_nonempty_and_roundtrips: delta must converge peer"
    );
}

// needed helper:
fn assert_delta_handles_bad_summary<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let bad = StateSummary::from(b"garbage".to_vec());
    let Ok(delta) = C::get_state_delta(cfg.params.clone(), state.clone(), bad) else {
        assert!(false, "get_state_delta with bad summary must not error");
        return;
    };
    let empty = (cfg.empty_state)();
    let result = C::update_state(cfg.params.clone(), empty, vec![UpdateData::Delta(delta)]);
    assert!(
        result.is_ok(),
        "delta_handles_bad_summary: must fallback to whole-state, not panic, got {result:?}"
    );
}

// needed helper:
fn assert_update_idempotent<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let data = (cfg.gen_update)(&state);
    let Some(once) = try_apply::<C>(cfg.params.clone(), state.clone(), vec![data.clone()]) else {
        assert!(false, "update_idempotent once failed");
        return;
    };
    let Some(twice) = try_apply::<C>(
        cfg.params.clone(),
        state.clone(),
        vec![data.clone(), data.clone()],
    ) else {
        assert!(false, "update_idempotent twice failed");
        return;
    };
    assert_eq!(
        once.as_ref(),
        twice.as_ref(),
        "update_idempotent: applying same update twice must equal once"
    );
    let Some(reapply) = try_apply::<C>(cfg.params.clone(), once.clone(), vec![data]) else {
        assert!(false, "reapply failed");
        return;
    };
    assert_eq!(
        once.as_ref(),
        reapply.as_ref(),
        "update_idempotent: reapplying after merge must be no-op"
    );
}

// needed helper:
fn assert_update_commutative<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let a = (cfg.gen_update)(&state);
    let b = (cfg.gen_update)(&state);
    let Some(ab) = try_apply::<C>(
        cfg.params.clone(),
        state.clone(),
        vec![a.clone(), b.clone()],
    ) else {
        assert!(false, "commutative ab failed");
        return;
    };
    let Some(ba) = try_apply::<C>(cfg.params.clone(), state, vec![b, a]) else {
        assert!(false, "commutative ba failed");
        return;
    };
    assert_eq!(
        ab.as_ref(),
        ba.as_ref(),
        "update_commutative: order of updates must not affect result"
    );
}

// needed helper:
fn assert_update_associative<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let a = (cfg.gen_update)(&state);
    let b = (cfg.gen_update)(&state);
    let c = (cfg.gen_update)(&state);
    let Some(ab) = try_apply::<C>(
        cfg.params.clone(),
        state.clone(),
        vec![a.clone(), b.clone()],
    ) else {
        assert!(false, "associative ab failed");
        return;
    };
    let Some(ab_c) = try_apply::<C>(cfg.params.clone(), ab, vec![c.clone()]) else {
        assert!(false, "associative ab_c failed");
        return;
    };
    let Some(a_bc) = try_apply::<C>(cfg.params.clone(), state, vec![a, b, c]) else {
        assert!(false, "associative a_bc failed");
        return;
    };
    assert_eq!(
        ab_c.as_ref(),
        a_bc.as_ref(),
        "update_associative: batch vs split apply must be equal"
    );
}

// needed helper:
fn assert_update_reads_data_not_state_plus1<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let data = (cfg.gen_update)(&state);
    let Some(once) = try_apply::<C>(cfg.params.clone(), state.clone(), vec![data.clone()]) else {
        assert!(false, "reads_data once failed");
        return;
    };
    let Some(twice) = try_apply::<C>(cfg.params.clone(), state, vec![data.clone(), data]) else {
        assert!(false, "reads_data twice failed");
        return;
    };
    assert_eq!(
        once.as_ref(),
        twice.as_ref(),
        "update_reads_data_not_state_plus1: must be max/union from data, not +1 from state"
    );
}

// needed helper:
fn assert_update_empty_and_unknown_noop<C: ContractInterface>(cfg: &suite_config::SuiteConfig) {
    let state = (cfg.gen_state)();
    let Some(empty) = try_apply::<C>(cfg.params.clone(), state.clone(), vec![]) else {
        assert!(false, "empty apply failed");
        return;
    };
    assert_eq!(
        empty.as_ref(),
        state.as_ref(),
        "update_empty_and_unknown_noop: empty data must be no-op"
    );
    let related_to = ContractInstanceId::new([9u8; 32]);
    let Ok(with_related) = C::update_state(
        cfg.params.clone(),
        state.clone(),
        vec![UpdateData::RelatedState {
            related_to,
            state: State::from(Vec::new()),
        }],
    ) else {
        assert!(false, "Related variant must not error");
        return;
    };
    let Some(related_state) = with_related.new_state else {
        assert!(false, "Related returned no state");
        return;
    };
    assert_eq!(
        related_state.as_ref(),
        state.as_ref(),
        "update_empty_and_unknown_noop: Related variant must be no-op"
    );
}

// needed helper:
fn assert_update_rejects_garbage_data_without_panic<C: ContractInterface>(
    cfg: &suite_config::SuiteConfig,
) {
    let state = (cfg.gen_state)();
    let bad = State::from(b"not-state".to_vec());
    let result = C::update_state(cfg.params.clone(), state, vec![UpdateData::State(bad)]);
    assert!(
        result.is_ok() || result.is_err(),
        "update_rejects_garbage: must not trap"
    );
}

fn try_apply<C: ContractInterface>(
    params: Parameters<'static>,
    state: State<'static>,
    datas: Vec<UpdateData<'static>>,
) -> Option<State<'static>> {
    let Ok(m) = C::update_state(params, state, datas) else {
        return None;
    };
    m.new_state
}

#[cfg(test)]
mod tests {
    use crate::suite_config::SuiteConfig;
    use freenet_stdlib::prelude::*;
    use std::collections::BTreeMap;

    struct DummyContract;

    type Slots = BTreeMap<u64, u64>;

    fn decode_slots(state: &[u8]) -> Result<Slots, ContractError> {
        bincode::deserialize::<Slots>(state).map_err(|_| ContractError::InvalidState)
    }

    fn encode_slots(slots: &Slots) -> State<'static> {
        State::from(bincode::serialize(slots).unwrap())
    }

    #[contract]
    impl ContractInterface for DummyContract {
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
            Ok(StateDelta::from(
                bincode::serialize(&delta).map_err(|e| ContractError::Deser(e.to_string()))?,
            ))
        }
    }

    fn slots(map: &[(u64, u64)]) -> State<'static> {
        let s: Slots = map.iter().copied().collect();
        encode_slots(&s)
    }

    fn cfg() -> SuiteConfig {
        SuiteConfig {
            params: Parameters::from(Vec::new()),
            gen_state: || slots(&[(0, 5), (1, 3)]),
            gen_update: |_| UpdateData::State(slots(&[(1, 7)])),
            gen_divergent_equal_total: Some(|| Some((slots(&[(0, 4), (1, 4)]), slots(&[(0, 8)])))),
            empty_state: || slots(&[]),
        }
    }

    #[test]
    fn validate_accepts_gen() {
        super::assert_validate_accepts_gen::<DummyContract>(&cfg());
    }

    #[test]
    fn validate_rejects_garbage() {
        super::assert_validate_rejects_garbage::<DummyContract>(&cfg());
    }

    #[test]
    fn summarize_deterministic() {
        super::assert_summarize_deterministic::<DummyContract>(&cfg());
    }

    #[test]
    fn summarize_detects_structural_divergence() {
        super::assert_summarize_detects_structural_divergence::<DummyContract>(&cfg());
    }

    #[test]
    fn delta_nonempty_and_roundtrips() {
        super::assert_delta_nonempty_and_roundtrips::<DummyContract>(&cfg());
    }

    #[test]
    fn delta_handles_bad_summary() {
        super::assert_delta_handles_bad_summary::<DummyContract>(&cfg());
    }

    #[test]
    fn update_idempotent() {
        super::assert_update_idempotent::<DummyContract>(&cfg());
    }

    #[test]
    fn update_commutative() {
        super::assert_update_commutative::<DummyContract>(&cfg());
    }

    #[test]
    fn update_associative() {
        super::assert_update_associative::<DummyContract>(&cfg());
    }

    #[test]
    fn update_reads_data_not_state_plus1() {
        super::assert_update_reads_data_not_state_plus1::<DummyContract>(&cfg());
    }

    #[test]
    fn update_empty_and_unknown_noop() {
        super::assert_update_empty_and_unknown_noop::<DummyContract>(&cfg());
    }

    #[test]
    fn update_rejects_garbage_data_without_panic() {
        super::assert_update_rejects_garbage_data_without_panic::<DummyContract>(&cfg());
    }

    #[test]
    fn test_usage() {
        super::run_suite::<DummyContract>(cfg());
    }
}
