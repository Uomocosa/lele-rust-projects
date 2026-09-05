use freenet_stdlib::prelude::*;

use crate::SuiteConfig;

pub fn run_suite<C: ContractInterface>(cfg: &SuiteConfig) {
    assert_validate_accepts_gen::<C>(cfg);
    assert_validate_rejects_garbage::<C>(cfg);
    assert_summarize_deterministic::<C>(cfg);
    assert_summarize_detects_structural_divergence::<C>(cfg);
    assert_delta_nonempty_and_roundtrips::<C>(cfg);
    assert_delta_handles_bad_summary::<C>(cfg);
    assert_self_delta_empty::<C>(cfg);
    assert_delta_equivalence::<C>(cfg);
    assert_update_idempotent::<C>(cfg);
    assert_update_commutative::<C>(cfg);
    assert_update_associative::<C>(cfg);
    assert_update_reads_data_not_state_plus1::<C>(cfg);
    assert_update_empty_and_unknown_noop::<C>(cfg);
    assert_update_state_and_delta::<C>(cfg);
    assert_update_rejects_garbage_data_without_panic::<C>(cfg);
}

// needed helper:
fn assert_validate_accepts_gen<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let result = C::validate_state(cfg.params.clone(), state, RelatedContracts::default());
    assert!(
        result.is_ok(),
        "validate_accepts_gen: gen_state should be valid"
    );
}

// needed helper:
fn assert_validate_rejects_garbage<C: ContractInterface>(cfg: &SuiteConfig) {
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
fn assert_summarize_deterministic<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let checked_1 = C::summarize_state(cfg.params.clone(), state.clone());
    assert!(
        checked_1.is_ok(),
        "summarize_deterministic: first summarize failed"
    );
    let Ok(a) = checked_1 else {
        return;
    };
    let checked_2 = C::summarize_state(cfg.params.clone(), state);
    assert!(
        checked_2.is_ok(),
        "summarize_deterministic: second summarize failed"
    );
    let Ok(b) = checked_2 else {
        return;
    };
    assert_eq!(
        a.as_ref(),
        b.as_ref(),
        "summarize_deterministic: same state must give same summary"
    );
}

// needed helper:
fn assert_summarize_detects_structural_divergence<C: ContractInterface>(cfg: &SuiteConfig) {
    let Some(divergent) = cfg.gen_divergent_equal_total else {
        return;
    };
    let Some((a, b)) = divergent() else {
        return;
    };
    let checked_3 = C::summarize_state(cfg.params.clone(), a);
    assert!(checked_3.is_ok(), "summarize divergent a failed");
    let Ok(sa) = checked_3 else {
        return;
    };
    let checked_4 = C::summarize_state(cfg.params.clone(), b);
    assert!(checked_4.is_ok(), "summarize divergent b failed");
    let Ok(sb) = checked_4 else {
        return;
    };
    assert_ne!(
        sa.as_ref(),
        sb.as_ref(),
        "summarize_detects_structural_divergence: equal totals with different shape must have different summaries (per-tag summary required)"
    );
}

// needed helper:
fn assert_delta_nonempty_and_roundtrips<C: ContractInterface>(cfg: &SuiteConfig) {
    let base = (cfg.gen_state)();
    let update = (cfg.gen_update)(&base);
    let checked_5 = try_apply::<C>(cfg.params.clone(), base.clone(), vec![update]);
    assert!(
        checked_5.is_some(),
        "delta_nonempty_and_roundtrips: apply ahead failed"
    );
    let Some(ahead) = checked_5 else {
        return;
    };
    if ahead.as_ref() == base.as_ref() {
        return;
    }
    let checked_6 = C::summarize_state(cfg.params.clone(), base.clone());
    assert!(checked_6.is_ok(), "summarize base for delta failed");
    let Ok(summary) = checked_6 else {
        return;
    };
    let checked_7 = C::get_state_delta(cfg.params.clone(), ahead.clone(), summary);
    assert!(checked_7.is_ok(), "get_state_delta failed");
    let Ok(delta) = checked_7 else {
        return;
    };
    assert!(
        !delta.as_ref().is_empty(),
        "delta_nonempty_and_roundtrips: empty delta disables anti-entropy"
    );
    let checked_8 = C::update_state(cfg.params.clone(), base, vec![UpdateData::Delta(delta)]);
    assert!(checked_8.is_ok(), "delta update failed");
    let Ok(merged) = checked_8 else {
        return;
    };
    let checked_9 = merged.new_state;
    assert!(checked_9.is_some(), "delta merge returned no state");
    let Some(merged_state) = checked_9 else {
        return;
    };
    assert_eq!(
        merged_state.as_ref(),
        ahead.as_ref(),
        "delta_nonempty_and_roundtrips: delta must converge peer"
    );
}

// needed helper:
fn assert_delta_handles_bad_summary<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let bad = StateSummary::from(b"garbage".to_vec());
    let checked_10 = C::get_state_delta(cfg.params.clone(), state.clone(), bad);
    assert!(
        checked_10.is_ok(),
        "get_state_delta with bad summary must not error"
    );
    let Ok(delta) = checked_10 else {
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
fn assert_update_idempotent<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let data = (cfg.gen_update)(&state);
    let checked_11 = try_apply::<C>(cfg.params.clone(), state.clone(), vec![data.clone()]);
    assert!(checked_11.is_some(), "update_idempotent once failed");
    let Some(once) = checked_11 else {
        return;
    };
    let checked_12 = try_apply::<C>(
        cfg.params.clone(),
        state.clone(),
        vec![data.clone(), data.clone()],
    );
    assert!(checked_12.is_some(), "update_idempotent twice failed");
    let Some(twice) = checked_12 else {
        return;
    };
    assert_eq!(
        once.as_ref(),
        twice.as_ref(),
        "update_idempotent: applying same update twice must equal once"
    );
    let checked_13 = try_apply::<C>(cfg.params.clone(), once.clone(), vec![data]);
    assert!(checked_13.is_some(), "reapply failed");
    let Some(reapply) = checked_13 else {
        return;
    };
    assert_eq!(
        once.as_ref(),
        reapply.as_ref(),
        "update_idempotent: reapplying after merge must be no-op"
    );
}

// needed helper:
fn assert_update_commutative<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let a = (cfg.gen_update)(&state);
    let b = (cfg.gen_update)(&state);
    let checked_14 = try_apply::<C>(
        cfg.params.clone(),
        state.clone(),
        vec![a.clone(), b.clone()],
    );
    assert!(checked_14.is_some(), "commutative ab failed");
    let Some(ab) = checked_14 else {
        return;
    };
    let checked_15 = try_apply::<C>(cfg.params.clone(), state, vec![b, a]);
    assert!(checked_15.is_some(), "commutative ba failed");
    let Some(ba) = checked_15 else {
        return;
    };
    assert_eq!(
        ab.as_ref(),
        ba.as_ref(),
        "update_commutative: order of updates must not affect result"
    );
}

// needed helper:
fn assert_update_associative<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let a = (cfg.gen_update)(&state);
    let b = (cfg.gen_update)(&state);
    let c = (cfg.gen_update)(&state);
    let checked_16 = try_apply::<C>(
        cfg.params.clone(),
        state.clone(),
        vec![a.clone(), b.clone()],
    );
    assert!(checked_16.is_some(), "associative ab failed");
    let Some(ab) = checked_16 else {
        return;
    };
    let checked_17 = try_apply::<C>(cfg.params.clone(), ab, vec![c.clone()]);
    assert!(checked_17.is_some(), "associative ab_c failed");
    let Some(ab_c) = checked_17 else {
        return;
    };
    let checked_18 = try_apply::<C>(cfg.params.clone(), state, vec![a, b, c]);
    assert!(checked_18.is_some(), "associative a_bc failed");
    let Some(a_bc) = checked_18 else {
        return;
    };
    assert_eq!(
        ab_c.as_ref(),
        a_bc.as_ref(),
        "update_associative: batch vs split apply must be equal"
    );
}

// needed helper:
fn assert_update_reads_data_not_state_plus1<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let data = (cfg.gen_update)(&state);
    let checked_19 = try_apply::<C>(cfg.params.clone(), state.clone(), vec![data.clone()]);
    assert!(checked_19.is_some(), "reads_data once failed");
    let Some(once) = checked_19 else {
        return;
    };
    let checked_20 = try_apply::<C>(cfg.params.clone(), state, vec![data.clone(), data]);
    assert!(checked_20.is_some(), "reads_data twice failed");
    let Some(twice) = checked_20 else {
        return;
    };
    assert_eq!(
        once.as_ref(),
        twice.as_ref(),
        "update_reads_data_not_state_plus1: must be max/union from data, not +1 from state"
    );
}

// needed helper:
fn assert_update_empty_and_unknown_noop<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let checked_21 = try_apply::<C>(cfg.params.clone(), state.clone(), vec![]);
    assert!(checked_21.is_some(), "empty apply failed");
    let Some(empty) = checked_21 else {
        return;
    };
    assert_eq!(
        empty.as_ref(),
        state.as_ref(),
        "update_empty_and_unknown_noop: empty data must be no-op"
    );
    let related_to = ContractInstanceId::new([9u8; 32]);
    let checked_22 = C::update_state(
        cfg.params.clone(),
        state.clone(),
        vec![UpdateData::RelatedState {
            related_to,
            state: State::from(Vec::new()),
        }],
    );
    assert!(checked_22.is_ok(), "Related variant must not error");
    let Ok(with_related) = checked_22 else {
        return;
    };
    let checked_23 = with_related.new_state;
    assert!(checked_23.is_some(), "Related returned no state");
    let Some(related_state) = checked_23 else {
        return;
    };
    assert_eq!(
        related_state.as_ref(),
        state.as_ref(),
        "update_empty_and_unknown_noop: Related variant must be no-op"
    );
}

// needed helper:
fn assert_self_delta_empty<C: ContractInterface>(cfg: &SuiteConfig) {
    let state = (cfg.gen_state)();
    let checked_24 = C::summarize_state(cfg.params.clone(), state.clone());
    assert!(checked_24.is_ok(), "self_delta_empty: summarize failed");
    let Ok(summary) = checked_24 else {
        return;
    };
    let checked_25 = C::get_state_delta(cfg.params.clone(), state.clone(), summary);
    assert!(
        checked_25.is_ok(),
        "self_delta_empty: get_state_delta failed"
    );
    let Ok(delta) = checked_25 else {
        return;
    };
    assert!(
        delta.as_ref().is_empty(),
        "self_delta_empty: delta against own summary must be empty (self_delta_empty #5072)"
    );
}

// needed helper:
fn assert_delta_equivalence<C: ContractInterface>(cfg: &SuiteConfig) {
    let base = (cfg.gen_state)();
    let update = (cfg.gen_update)(&base);
    let checked_26 = try_apply::<C>(cfg.params.clone(), base.clone(), vec![update]);
    assert!(
        checked_26.is_some(),
        "delta_equivalence: ahead apply failed"
    );
    let Some(ahead) = checked_26 else {
        return;
    };
    if ahead.as_ref() == base.as_ref() {
        return;
    }
    let checked_27 = C::summarize_state(cfg.params.clone(), base.clone());
    assert!(
        checked_27.is_ok(),
        "delta_equivalence: summarize base failed"
    );
    let Ok(summary) = checked_27 else {
        return;
    };
    let checked_28 = C::get_state_delta(cfg.params.clone(), ahead.clone(), summary);
    assert!(
        checked_28.is_ok(),
        "delta_equivalence: get_state_delta failed"
    );
    let Ok(delta) = checked_28 else {
        return;
    };
    assert!(
        !delta.as_ref().is_empty(),
        "delta_equivalence: delta should be non-empty"
    );
    let checked_29 = try_apply::<C>(
        cfg.params.clone(),
        base.clone(),
        vec![UpdateData::Delta(delta)],
    );
    assert!(
        checked_29.is_some(),
        "delta_equivalence: via_delta apply failed"
    );
    let Some(via_delta) = checked_29 else {
        return;
    };
    let checked_35 = try_apply::<C>(
        cfg.params.clone(),
        base,
        vec![UpdateData::State(ahead.clone())],
    );
    assert!(
        checked_35.is_some(),
        "delta_equivalence: via_state apply failed"
    );
    let Some(via_state) = checked_35 else {
        return;
    };
    assert_eq!(
        via_delta.as_ref(),
        via_state.as_ref(),
        "delta_equivalence: delta and whole-state merge must be equivalent (R ∪ (S\\R) == R ∪ S)"
    );
    assert_eq!(
        via_delta.as_ref(),
        ahead.as_ref(),
        "delta_equivalence: result must equal ahead"
    );
}

// needed helper:
fn assert_update_state_and_delta<C: ContractInterface>(cfg: &SuiteConfig) {
    let base = (cfg.gen_state)();
    let update_a = (cfg.gen_update)(&base);
    let checked_30 = try_apply::<C>(cfg.params.clone(), base.clone(), vec![update_a.clone()]);
    assert!(
        checked_30.is_some(),
        "state_and_delta: first update apply failed"
    );
    let Some(ahead_a) = checked_30 else {
        return;
    };
    if ahead_a.as_ref() == base.as_ref() {
        return;
    }
    let delta_bytes = match &update_a {
        UpdateData::State(s) => s.as_ref().to_vec(),
        UpdateData::Delta(d) => d.as_ref().to_vec(),
        _ => return,
    };
    let state_bytes = ahead_a.as_ref().to_vec();
    let checked_31 = C::update_state(
        cfg.params.clone(),
        base.clone(),
        vec![UpdateData::StateAndDelta {
            state: State::from(Vec::new()),
            delta: StateDelta::from(Vec::new()),
        }],
    );
    assert!(
        checked_31.is_ok(),
        "state_and_delta: empty StateAndDelta must not error"
    );
    let Ok(empty_apply) = checked_31 else {
        return;
    };
    let checked_32 = empty_apply.new_state;
    assert!(
        checked_32.is_some(),
        "state_and_delta: empty returned no state"
    );
    let Some(empty_state) = checked_32 else {
        return;
    };
    assert_eq!(
        empty_state.as_ref(),
        base.as_ref(),
        "state_and_delta: empty StateAndDelta must be no-op"
    );
    let checked_33 = C::update_state(
        cfg.params.clone(),
        base.clone(),
        vec![UpdateData::StateAndDelta {
            state: State::from(state_bytes),
            delta: StateDelta::from(delta_bytes),
        }],
    );
    assert!(checked_33.is_ok(), "state_and_delta: combined apply failed");
    let Ok(both) = checked_33 else {
        return;
    };
    let checked_34 = both.new_state;
    assert!(
        checked_34.is_some(),
        "state_and_delta: combined returned no state"
    );
    let Some(combined) = checked_34 else {
        return;
    };
    assert!(
        combined.as_ref().len() >= base.as_ref().len(),
        "state_and_delta: combined must not shrink base"
    );
}

// needed helper:
fn assert_update_rejects_garbage_data_without_panic<C: ContractInterface>(cfg: &SuiteConfig) {
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
    use crate::SuiteConfig;
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
                match update {
                    UpdateData::State(s) => {
                        if s.as_ref().is_empty() {
                            continue;
                        }
                        let incoming =
                            decode_slots(s.as_ref()).map_err(|_| ContractError::InvalidUpdate)?;
                        for (tag, value) in incoming {
                            let entry = current.entry(tag).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    }
                    UpdateData::Delta(d) => {
                        if d.as_ref().is_empty() {
                            continue;
                        }
                        let incoming =
                            decode_slots(d.as_ref()).map_err(|_| ContractError::InvalidUpdate)?;
                        for (tag, value) in incoming {
                            let entry = current.entry(tag).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    }
                    UpdateData::StateAndDelta { state: s, delta: d } => {
                        if !s.as_ref().is_empty() {
                            let incoming = decode_slots(s.as_ref())
                                .map_err(|_| ContractError::InvalidUpdate)?;
                            for (tag, value) in incoming {
                                let entry = current.entry(tag).or_insert(0);
                                *entry = (*entry).max(value);
                            }
                        }
                        if !d.as_ref().is_empty() {
                            let incoming = decode_slots(d.as_ref())
                                .map_err(|_| ContractError::InvalidUpdate)?;
                            for (tag, value) in incoming {
                                let entry = current.entry(tag).or_insert(0);
                                *entry = (*entry).max(value);
                            }
                        }
                    }
                    _ => {}
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
            if delta.is_empty() {
                return Ok(StateDelta::from(Vec::new()));
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
    fn self_delta_empty() {
        super::assert_self_delta_empty::<DummyContract>(&cfg());
    }

    #[test]
    fn delta_equivalence() {
        super::assert_delta_equivalence::<DummyContract>(&cfg());
    }

    #[test]
    fn state_and_delta() {
        super::assert_update_state_and_delta::<DummyContract>(&cfg());
    }

    #[test]
    fn test_usage() {
        super::run_suite::<DummyContract>(&cfg());
    }
}
