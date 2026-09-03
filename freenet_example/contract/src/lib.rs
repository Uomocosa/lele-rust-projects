#![allow(unexpected_cfgs)]

use std::collections::{BTreeMap, BTreeSet};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
struct GlobalCounterContract;

type Pubkey = [u8; 32];
type Slots = BTreeMap<Pubkey, u64>;
const ACTIVE_CAP: usize = 32;

#[derive(Serialize, Deserialize)]
struct SignedSlots {
    slots: BTreeMap<Pubkey, u64>,
    sigs: BTreeMap<Pubkey, Vec<u8>>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct WindowedState {
    base: u64,
    slots: BTreeMap<Pubkey, u64>,
    tombstones: BTreeSet<Pubkey>,
}

fn decode_windowed(state: &[u8]) -> Result<WindowedState, ContractError> {
    if state.is_empty() {
        return Err(ContractError::InvalidState);
    }
    if let Ok(ws) = bincode::deserialize::<WindowedState>(state) {
        return Ok(ws);
    }
    let slots = bincode::deserialize::<Slots>(state).map_err(|_| ContractError::InvalidState)?;
    Ok(WindowedState {
        base: 0,
        slots,
        tombstones: BTreeSet::new(),
    })
}

fn encode_windowed(ws: &WindowedState) -> State<'static> {
    State::from(bincode::serialize(ws).expect("serialize windowed"))
}

fn decode_params(params: &[u8]) -> BTreeSet<Pubkey> {
    if params.is_empty() {
        return BTreeSet::new();
    }
    bincode::deserialize::<BTreeSet<Pubkey>>(params).unwrap_or_default()
}

#[allow(dead_code)]
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

fn fold_excess(ws: &mut WindowedState) {
    while ws.slots.len() > ACTIVE_CAP {
        let Some((k, v)) = ws.slots.pop_first() else {
            break;
        };
        ws.base = ws.base.saturating_add(v);
        ws.tombstones.insert(k);
    }
}

#[contract]
impl ContractInterface for GlobalCounterContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        decode_windowed(state.as_ref()).map(|_| ValidateResult::Valid)
    }

    fn update_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut current = decode_windowed(state.as_ref()).unwrap_or_default();
        let allow_list = decode_params(parameters.as_ref());
        let use_allow = !allow_list.is_empty();
        let window: u64 = 1;
        for update in data {
            match update {
                UpdateData::State(s) => {
                    let as_ws = bincode::deserialize::<WindowedState>(s.as_ref()).ok();
                    let as_signed = bincode::deserialize::<SignedSlots>(s.as_ref()).ok();
                    let ws_roundtrip = as_ws.as_ref().map(|ws| bincode::serialize(ws).map(|b| b.len() == s.as_ref().len()).unwrap_or(false)).unwrap_or(false);
                    let signed_roundtrip = as_signed.as_ref().map(|ss| bincode::serialize(ss).map(|b| b.len() == s.as_ref().len()).unwrap_or(false)).unwrap_or(false);
                    if ws_roundtrip && !signed_roundtrip {
                        let ws = as_ws.unwrap();
                        for (pubkey, value) in ws.slots {
                            if current.tombstones.contains(&pubkey) { continue; }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur { continue; }
                            if value > cur.saturating_add(window) { continue; }
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                        if ws.base > current.base { current.base = current.base.max(ws.base); }
                        for t in ws.tombstones { current.tombstones.insert(t); current.slots.remove(&t); }
                    } else if signed_roundtrip {
                        let signed = as_signed.unwrap();
                        for (pubkey, value) in signed.slots {
                            if current.tombstones.contains(&pubkey) { continue; }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur { continue; }
                            if value > cur.saturating_add(window) { continue; }
                            if use_allow && !allow_list.contains(&pubkey) { continue; }
                            let Some(sig) = signed.sigs.get(&pubkey) else { continue; };
                            if !verify_sig(&pubkey, value, sig) { continue; }
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    } else if let Some(ws) = as_ws {
                        for (pubkey, value) in ws.slots {
                            if current.tombstones.contains(&pubkey) { continue; }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur { continue; }
                            if value > cur.saturating_add(window) { continue; }
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                        if ws.base > current.base { current.base = current.base.max(ws.base); }
                        for t in ws.tombstones { current.tombstones.insert(t); current.slots.remove(&t); }
                    } else if let Some(signed) = as_signed {
                        for (pubkey, value) in signed.slots {
                            if current.tombstones.contains(&pubkey) {
                                continue;
                            }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
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
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    } else if let Ok(incoming) = bincode::deserialize::<Slots>(s.as_ref()) {
                        for (pubkey, value) in incoming {
                            if current.tombstones.contains(&pubkey) {
                                continue;
                            }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur {
                                continue;
                            }
                            if value > cur.saturating_add(window) {
                                continue;
                            }
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    }
                }
                UpdateData::Delta(d) => {
                    if let Ok(ws) = bincode::deserialize::<WindowedState>(d.as_ref()) {
                        for (pubkey, value) in ws.slots {
                            if current.tombstones.contains(&pubkey) {
                                continue;
                            }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur {
                                continue;
                            }
                            if value > cur.saturating_add(window) {
                                continue;
                            }
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    } else if let Ok(incoming) = bincode::deserialize::<Slots>(d.as_ref()) {
                        for (pubkey, value) in incoming {
                            if current.tombstones.contains(&pubkey) {
                                continue;
                            }
                            let cur = current.slots.get(&pubkey).copied().unwrap_or(0);
                            if value <= cur {
                                continue;
                            }
                            if value > cur.saturating_add(window) {
                                continue;
                            }
                            let entry = current.slots.entry(pubkey).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    }
                }
                _ => continue,
            }
        }
        fold_excess(&mut current);
        let new_state = encode_windowed(&current);
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let ws = decode_windowed(state.as_ref())?;
        let summary = bincode::serialize(&ws).map_err(|e| ContractError::Deser(e.to_string()))?;
        Ok(StateSummary::from(summary))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        let ours = decode_windowed(state.as_ref())?;
        let theirs: WindowedState = match bincode::deserialize(summary.as_ref()) {
            Ok(s) => s,
            Err(_) => return Ok(StateDelta::from(state.as_ref().to_vec())),
        };
        let mut delta = WindowedState {
            base: 0,
            slots: BTreeMap::new(),
            tombstones: BTreeSet::new(),
        };
        if ours.base > theirs.base {
            delta.base = ours.base;
        }
        for (pubkey, value) in &ours.slots {
            if theirs.slots.get(pubkey).copied().unwrap_or(0) < *value {
                delta.slots.insert(*pubkey, *value);
            }
        }
        for t in &ours.tombstones {
            if !theirs.tombstones.contains(t) {
                delta.tombstones.insert(*t);
            }
        }
        if delta.base == 0 && delta.slots.is_empty() && delta.tombstones.is_empty() {
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

    fn windowed_state(map: &[(u64, u64)], base: u64) -> State<'static> {
        let mut ws = WindowedState {
            base,
            slots: BTreeMap::new(),
            tombstones: BTreeSet::new(),
        };
        for (t, v) in map {
            ws.slots.insert(pubkey_for_tag(*t), *v);
        }
        encode_windowed(&ws)
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
        let state = windowed_state(&[(0, 5), (1, 3)], 0);
        let related = RelatedContracts::default();
        assert!(GlobalCounterContract::validate_state(params(), state.clone(), related).is_ok());

        let update = vec![UpdateData::State(signed_state(&[(1, 4)]))];
        let result = GlobalCounterContract::update_state(params(), state.clone(), update).unwrap();
        let next_state = result.unwrap_valid();

        let summary = GlobalCounterContract::summarize_state(params(), next_state.clone()).unwrap();
        let _delta = GlobalCounterContract::get_state_delta(params(), next_state, summary).unwrap();
    }

    #[test]
    fn harness_candidate_pubkey() {
        let cfg = freenet_contract_harness::SuiteConfig {
            params: params_for_tags(&[0, 1]),
            gen_state: || windowed_state(&[(0, 5), (1, 3)], 0),
            gen_update: |_| UpdateData::State(signed_state(&[(1, 4)])),
            gen_divergent_equal_total: Some(|| {
                Some((
                    windowed_state(&[(0, 4), (1, 4)], 0),
                    windowed_state(&[(0, 8)], 0),
                ))
            }),
            empty_state: || windowed_state(&[], 0),
        };
        freenet_contract_harness::run_suite::<GlobalCounterContract>(cfg);
    }

    #[test]
    fn window_blocks_large_jump() {
        let state = windowed_state(&[(0, 5)], 0);
        let p = params_for_tags(&[0]);
        let max_jump = vec![UpdateData::State(signed_state(&[(0, 1_000_000)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), max_jump).unwrap();
        let next = result.unwrap_valid();
        let decoded = decode_windowed(next.as_ref()).unwrap();
        let pk = pubkey_for_tag(0);
        assert_eq!(decoded.slots.get(&pk), Some(&5), "MAX jump should be blocked");

        let ok_step = vec![UpdateData::State(signed_state(&[(0, 6)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), ok_step).unwrap();
        let next = result.unwrap_valid();
        let decoded = decode_windowed(next.as_ref()).unwrap();
        assert_eq!(decoded.slots.get(&pk), Some(&6), "value +1 should be accepted");

        let plus_two = vec![UpdateData::State(signed_state(&[(0, 7)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), plus_two).unwrap();
        let next = result.unwrap_valid();
        let decoded = decode_windowed(next.as_ref()).unwrap();
        assert_eq!(decoded.slots.get(&pk), Some(&5), "value +2 should be rejected");
    }

    #[test]
    fn windowed_fold() {
        let mut map = Vec::new();
        for i in 0..(ACTIVE_CAP as u64 + 5) {
            map.push((i, i + 1));
        }
        let mut ws = WindowedState {
            base: 0,
            slots: map.iter().map(|(t, v)| (pubkey_for_tag(*t), *v)).collect(),
            tombstones: BTreeSet::new(),
        };
        fold_excess(&mut ws);
        assert_eq!(ws.slots.len(), ACTIVE_CAP);
        assert!(ws.base > 0);
        assert_eq!(ws.tombstones.len(), 5);
    }
}
