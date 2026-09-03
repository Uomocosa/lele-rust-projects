#![allow(unexpected_cfgs)]

use std::collections::{BTreeMap, BTreeSet};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
struct GlobalCounterContract;

type Pubkey = [u8; 32];
type Slots = BTreeMap<Pubkey, u64>;

#[derive(Serialize, Deserialize)]
struct SignedSlots {
    slots: BTreeMap<Pubkey, u64>,
    sigs: BTreeMap<Pubkey, Vec<u8>>,
}

fn decode_slots(state: &[u8]) -> Result<Slots, ContractError> {
    bincode::deserialize::<Slots>(state).map_err(|_| ContractError::InvalidState)
}

fn encode_slots(slots: &Slots) -> State<'static> {
    State::from(bincode::serialize(slots).expect("serialize slots"))
}

fn decode_params(params: &[u8]) -> BTreeSet<Pubkey> {
    if params.is_empty() {
        return BTreeSet::new();
    }
    bincode::deserialize::<BTreeSet<Pubkey>>(params).unwrap_or_default()
}

fn decode_signed_slots(bytes: &[u8]) -> Result<SignedSlots, ContractError> {
    bincode::deserialize::<SignedSlots>(bytes).map_err(|_| ContractError::InvalidUpdate)
}

fn verify_sig(pubkey: &Pubkey, value: u64, sig_bytes: &[u8]) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(pubkey) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(sig_bytes) else {
        return false;
    };
    let msg = bincode::serialize(&(pubkey, value)).unwrap_or_default();
    vk.verify(&msg, &sig).is_ok()
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
        parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut current = decode_slots(state.as_ref()).unwrap_or_default();
        let allow_list = decode_params(parameters.as_ref());
        let use_allow = !allow_list.is_empty();
        let window: u64 = 1;
        for update in data {
            match update {
                UpdateData::State(s) => {
                    if let Ok(signed) = decode_signed_slots(s.as_ref()) {
                        for (pubkey, value) in signed.slots {
                            let cur = current.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur {
                                continue;
                            }
                            if value > cur.saturating_add(window) {
                                continue;
                            }
                            if use_allow && !allow_list.contains(&pubkey) {
                                continue;
                            }
                            let Some(sig) = signed.sigs.get(&pubkey) else {
                                continue;
                            };
                            if !verify_sig(&pubkey, value, sig) {
                                continue;
                            }
                            let entry = current.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    } else if let Ok(incoming) = decode_slots(s.as_ref()) {
                        for (pubkey, value) in incoming {
                            let cur = current.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur {
                                continue;
                            }
                            if value > cur.saturating_add(window) {
                                continue;
                            }
                            let entry = current.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    }
                }
                UpdateData::Delta(d) => {
                    let Ok(incoming) = decode_slots(d.as_ref()) else {
                        continue;
                    };
                    for (pubkey, value) in incoming {
                        let cur = current.get(&pubkey).copied().unwrap_or(0);
                        if value <= cur {
                            continue;
                        }
                        if value > cur.saturating_add(window) {
                            continue;
                        }
                        let entry = current.entry(pubkey).or_insert(0);
                        *entry = (*entry).max(value);
                    }
                }
                _ => continue,
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
        for (pubkey, value) in &ours {
            if theirs.get(pubkey).copied().unwrap_or(0) < *value {
                delta.insert(*pubkey, *value);
            }
        }
        if delta.is_empty() {
            return Ok(StateDelta::from(Vec::new()));
        }
        Ok(StateDelta::from(bincode::serialize(&delta).map_err(
            |e| ContractError::Deser(e.to_string()),
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn pubkey_for_tag(tag: u64) -> Pubkey {
        let mut seed = [0u8; 32];
        seed[0..8].copy_from_slice(&tag.to_le_bytes());
        let sk = SigningKey::from_bytes(&seed);
        let vk = VerifyingKey::from(&sk);
        *vk.as_bytes()
    }

    fn slots(map: &[(u64, u64)]) -> State<'static> {
        let s: Slots = map.iter().map(|(t, v)| (pubkey_for_tag(*t), *v)).collect();
        encode_slots(&s)
    }

    fn params_for_tags(tags: &[u64]) -> Parameters<'static> {
        let set: BTreeSet<Pubkey> = tags.iter().map(|t| pubkey_for_tag(*t)).collect();
        Parameters::from(bincode::serialize(&set).unwrap())
    }

    fn signed_state(slots_map: &[(u64, u64)]) -> State<'static> {
        let mut m = BTreeMap::new();
        let mut sigs = BTreeMap::new();
        for &(tag, value) in slots_map {
            let pk = pubkey_for_tag(tag);
            let mut seed = [0u8; 32];
            seed[0..8].copy_from_slice(&tag.to_le_bytes());
            let sk = SigningKey::from_bytes(&seed);
            let msg = bincode::serialize(&(pk, value)).unwrap();
            let sig = sk.sign(&msg);
            m.insert(pk, value);
            sigs.insert(pk, sig.to_bytes().to_vec());
        }
        let ss = SignedSlots { slots: m, sigs };
        State::from(bincode::serialize(&ss).unwrap())
    }

    fn params() -> Parameters<'static> {
        params_for_tags(&[0, 1])
    }

    #[test]
    fn test_usage() {
        let state = slots(&[(0, 5), (1, 3)]);
        let related = RelatedContracts::default();
        assert!(GlobalCounterContract::validate_state(params(), state.clone(), related).is_ok());

        let update = vec![UpdateData::State(signed_state(&[(1, 7)]))];
        let result = GlobalCounterContract::update_state(params(), state.clone(), update).unwrap();
        let next_state = result.unwrap_valid();

        let summary = GlobalCounterContract::summarize_state(params(), next_state.clone()).unwrap();
        let _delta = GlobalCounterContract::get_state_delta(params(), next_state, summary).unwrap();
    }

    #[test]
    fn harness_candidate_pubkey() {
        let cfg = freenet_contract_harness::SuiteConfig {
            params: params_for_tags(&[0, 1]),
            gen_state: || slots(&[(0, 5), (1, 3)]),
            gen_update: |_| UpdateData::State(signed_state(&[(1, 4)])),
            gen_divergent_equal_total: Some(|| Some((slots(&[(0, 4), (1, 4)]), slots(&[(0, 8)])))),
            empty_state: || slots(&[]),
        };
        freenet_contract_harness::run_suite::<GlobalCounterContract>(cfg);
    }

    #[test]
    fn window_blocks_large_jump() {
        let state = slots(&[(0, 5)]);
        let p = params_for_tags(&[0]);
        let max_jump = vec![UpdateData::State(signed_state(&[(0, 1_000_000)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), max_jump).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Slots>(next.as_ref()).unwrap();
        let pk = pubkey_for_tag(0);
        assert_eq!(decoded.get(&pk), Some(&5), "MAX jump should be blocked by window");

        let ok_step = vec![UpdateData::State(signed_state(&[(0, 6)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), ok_step).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Slots>(next.as_ref()).unwrap();
        assert_eq!(decoded.get(&pk), Some(&6), "value within window +1 should be accepted");

        let plus_two = vec![UpdateData::State(signed_state(&[(0, 7)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), plus_two).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Slots>(next.as_ref()).unwrap();
        assert_eq!(decoded.get(&pk), Some(&5), "value +2 should be rejected with window 1");

        let bad_sig = {
            let pk = pubkey_for_tag(0);
            let mut ss = SignedSlots {
                slots: BTreeMap::from([(pk, 6)]),
                sigs: BTreeMap::new(),
            };
            ss.sigs.insert(pk, vec![0u8; 64]);
            State::from(bincode::serialize(&ss).unwrap())
        };
        let result = GlobalCounterContract::update_state(p, state, vec![UpdateData::State(bad_sig)]).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Slots>(next.as_ref()).unwrap();
        assert_eq!(decoded.get(&pubkey_for_tag(0)), Some(&5), "bad sig should be rejected");
    }
}
