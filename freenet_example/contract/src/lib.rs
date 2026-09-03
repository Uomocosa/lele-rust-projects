#![allow(unexpected_cfgs)]

use std::collections::{BTreeMap, BTreeSet};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
struct GlobalCounterContract;

type Pubkey = [u8; 32];
type ShardId = u8;
const NUM_SHARDS: u8 = 16;
type Shards = BTreeMap<ShardId, u64>;

#[derive(Serialize, Deserialize)]
struct SignedShards {
    shards: BTreeMap<ShardId, u64>,
    sigs: BTreeMap<ShardId, Vec<u8>>,
}

fn pubkey_for_shard(shard: ShardId) -> Pubkey {
    let mut seed = [0u8; 32];
    seed[0] = shard;
    seed[1] = 0xC6;
    let sk = ed25519_dalek::SigningKey::from_bytes(&seed);
    let vk = VerifyingKey::from(&sk);
    *vk.as_bytes()
}

fn decode_shards(state: &[u8]) -> Result<Shards, ContractError> {
    bincode::deserialize::<Shards>(state).map_err(|_| ContractError::InvalidState)
}

fn encode_shards(shards: &Shards) -> State<'static> {
    State::from(bincode::serialize(shards).expect("serialize shards"))
}

fn decode_params(params: &[u8]) -> BTreeSet<Pubkey> {
    if params.is_empty() {
        return BTreeSet::new();
    }
    bincode::deserialize::<BTreeSet<Pubkey>>(params).unwrap_or_default()
}

fn decode_signed_shards(bytes: &[u8]) -> Result<SignedShards, ContractError> {
    bincode::deserialize::<SignedShards>(bytes).map_err(|_| ContractError::InvalidUpdate)
}

fn verify_shard_sig(shard: ShardId, value: u64, sig_bytes: &[u8]) -> bool {
    let pubkey = pubkey_for_shard(shard);
    let Ok(vk) = VerifyingKey::from_bytes(&pubkey) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(sig_bytes) else {
        return false;
    };
    let msg = bincode::serialize(&(shard, value)).unwrap_or_default();
    vk.verify(&msg, &sig).is_ok()
}

#[contract]
impl ContractInterface for GlobalCounterContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        let shards = decode_shards(state.as_ref())?;
        if shards.len() > usize::from(NUM_SHARDS) {
            return Err(ContractError::InvalidState);
        }
        for k in shards.keys() {
            if *k >= NUM_SHARDS {
                return Err(ContractError::InvalidState);
            }
        }
        Ok(ValidateResult::Valid)
    }

    fn update_state(
        parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut current = decode_shards(state.as_ref()).unwrap_or_default();
        let allow_list = decode_params(parameters.as_ref());
        let use_allow = !allow_list.is_empty();
        let window: u64 = 1;
        for update in data {
            match update {
                UpdateData::State(s) => {
                    if let Ok(signed) = decode_signed_shards(s.as_ref()) {
                        for (shard, value) in signed.shards {
                            if shard >= NUM_SHARDS { continue; }
                            let cur = current.get(&shard).copied().unwrap_or(0);
                            if value <= cur { continue; }
                            if value > cur.saturating_add(window) { continue; }
                            let pubkey = pubkey_for_shard(shard);
                            if use_allow && !allow_list.contains(&pubkey) { continue; }
                            let Some(sig) = signed.sigs.get(&shard) else { continue; };
                            if !verify_shard_sig(shard, value, sig) { continue; }
                            let entry = current.entry(shard).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    } else if let Ok(incoming) = decode_shards(s.as_ref()) {
                        for (shard, value) in incoming {
                            if shard >= NUM_SHARDS { continue; }
                            let cur = current.get(&shard).copied().unwrap_or(0);
                            if value <= cur { continue; }
                            if value > cur.saturating_add(window) { continue; }
                            let pubkey = pubkey_for_shard(shard);
                            if use_allow && !allow_list.contains(&pubkey) { continue; }
                            let entry = current.entry(shard).or_insert(0);
                            *entry = (*entry).max(value);
                        }
                    }
                }
                UpdateData::Delta(d) => {
                    let Ok(incoming) = decode_shards(d.as_ref()) else { continue; };
                    for (shard, value) in incoming {
                        if shard >= NUM_SHARDS { continue; }
                        let cur = current.get(&shard).copied().unwrap_or(0);
                        if value <= cur { continue; }
                        if value > cur.saturating_add(window) { continue; }
                        let pubkey = pubkey_for_shard(shard);
                        if use_allow && !allow_list.contains(&pubkey) { continue; }
                        let entry = current.entry(shard).or_insert(0);
                        *entry = (*entry).max(value);
                    }
                }
                _ => continue,
            }
        }
        let new_state = encode_shards(&current);
        Ok(UpdateModification::valid(new_state))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let shards = decode_shards(state.as_ref())?;
        let summary = bincode::serialize(&shards).map_err(|e| ContractError::Deser(e.to_string()))?;
        Ok(StateSummary::from(summary))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        let ours = decode_shards(state.as_ref())?;
        let theirs: Shards = match bincode::deserialize(summary.as_ref()) {
            Ok(s) => s,
            Err(_) => return Ok(StateDelta::from(state.as_ref().to_vec())),
        };
        let mut delta = Shards::new();
        for (shard, value) in &ours {
            if theirs.get(shard).copied().unwrap_or(0) < *value {
                delta.insert(*shard, *value);
            }
        }
        if delta.is_empty() {
            return Ok(StateDelta::from(Vec::new()));
        }
        Ok(StateDelta::from(bincode::serialize(&delta).map_err(|e| ContractError::Deser(e.to_string()))?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn shards(map: &[(u8, u64)]) -> State<'static> {
        let s: Shards = map.iter().copied().collect();
        encode_shards(&s)
    }

    fn params_for_shards(ids: &[u8]) -> Parameters<'static> {
        let set: BTreeSet<Pubkey> = ids.iter().map(|id| pubkey_for_shard(*id)).collect();
        Parameters::from(bincode::serialize(&set).unwrap())
    }

    fn signed_shards(map: &[(u8, u64)]) -> State<'static> {
        let mut m = BTreeMap::new();
        let mut sigs = BTreeMap::new();
        for &(shard, value) in map {
            let mut seed = [0u8; 32];
            seed[0] = shard;
            seed[1] = 0xC6;
            let sk = SigningKey::from_bytes(&seed);
            let msg = bincode::serialize(&(shard, value)).unwrap();
            let sig = sk.sign(&msg);
            m.insert(shard, value);
            sigs.insert(shard, sig.to_bytes().to_vec());
        }
        let ss = SignedShards { shards: m, sigs };
        State::from(bincode::serialize(&ss).unwrap())
    }

    fn params() -> Parameters<'static> { params_for_shards(&[0, 1]) }

    #[test]
    fn test_usage() {
        let state = shards(&[(0, 5), (1, 3)]);
        let related = RelatedContracts::default();
        assert!(GlobalCounterContract::validate_state(params(), state.clone(), related).is_ok());
        let update = vec![UpdateData::State(signed_shards(&[(1, 4)]))];
        let result = GlobalCounterContract::update_state(params(), state.clone(), update).unwrap();
        let next_state = result.unwrap_valid();
        let summary = GlobalCounterContract::summarize_state(params(), next_state.clone()).unwrap();
        let _delta = GlobalCounterContract::get_state_delta(params(), next_state, summary).unwrap();
    }

    #[test]
    fn harness_candidate_pubkey() {
        let cfg = freenet_contract_harness::SuiteConfig {
            params: params_for_shards(&[0, 1]),
            gen_state: || shards(&[(0, 5), (1, 3)]),
            gen_update: |_| UpdateData::State(signed_shards(&[(1, 4)])),
            gen_divergent_equal_total: Some(|| Some((shards(&[(0, 4), (1, 4)]), shards(&[(2, 8)])))),
            empty_state: || shards(&[]),
        };
        freenet_contract_harness::run_suite::<GlobalCounterContract>(cfg);
    }

    #[test]
    fn window_blocks_large_jump() {
        let state = shards(&[(0, 5)]);
        let p = params_for_shards(&[0]);
        let max_jump = vec![UpdateData::State(signed_shards(&[(0, 1_000_000)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), max_jump).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Shards>(next.as_ref()).unwrap();
        assert_eq!(decoded.get(&0), Some(&5));
        let ok_step = vec![UpdateData::State(signed_shards(&[(0, 6)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), ok_step).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Shards>(next.as_ref()).unwrap();
        assert_eq!(decoded.get(&0), Some(&6));
        let plus_two = vec![UpdateData::State(signed_shards(&[(0, 7)]))];
        let result = GlobalCounterContract::update_state(p.clone(), state.clone(), plus_two).unwrap();
        let next = result.unwrap_valid();
        let decoded = bincode::deserialize::<Shards>(next.as_ref()).unwrap();
        assert_eq!(decoded.get(&0), Some(&5));
    }

    #[test]
    fn sharded_bounded() {
        let state = shards(&[(0, 1), (15, 10)]);
        let p = params_for_shards(&[0]);
        assert!(GlobalCounterContract::validate_state(p, state, RelatedContracts::default()).is_ok());
        let mut big: Shards = BTreeMap::new();
        for i in 0..NUM_SHARDS { big.insert(i, u64::from(i)*10); }
        let encoded = encode_shards(&big);
        let decoded = bincode::deserialize::<Shards>(encoded.as_ref()).unwrap();
        assert_eq!(decoded.len(), usize::from(NUM_SHARDS));
    }
}
