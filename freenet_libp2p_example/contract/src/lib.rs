#![allow(unexpected_cfgs)]

use std::collections::BTreeMap;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
struct LetterContract;

type Pubkey = [u8; 32];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PeerRecord {
    peer_id: Vec<u8>,
    addrs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ChainEntry {
    author: Pubkey,
    prev: u8,
    next: u8,
    sig: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
struct StateData {
    peers: BTreeMap<Pubkey, PeerRecord>,
    chain: BTreeMap<u64, ChainEntry>,
    sigs: BTreeMap<Pubkey, Vec<u8>>,
}

fn decode_state(bytes: &[u8]) -> Result<StateData, ContractError> {
    if bytes.is_empty() {
        return Ok(StateData::default());
    }
    bincode::deserialize::<StateData>(bytes).map_err(|_| ContractError::InvalidState)
}

fn encode_state(data: &StateData) -> State<'static> {
    State::from(bincode::serialize(data).expect("serialize state"))
}

fn verify_peer_sig(pubkey: &Pubkey, record: &PeerRecord, sig: &[u8]) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(pubkey) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(sig) else {
        return false;
    };
    let msg = bincode::serialize(&(pubkey, &record.peer_id, &record.addrs)).unwrap_or_default();
    vk.verify(&msg, &sig).is_ok()
}

fn verify_chain_sig(entry: &ChainEntry, seq: u64) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(&entry.author) else {
        return false;
    };
    let Ok(sig) = Signature::from_slice(&entry.sig) else {
        return false;
    };
    let msg =
        bincode::serialize(&(seq, entry.author, entry.prev, entry.next)).unwrap_or_default();
    vk.verify(&msg, &sig).is_ok()
}

const MAX_CHAIN: usize = 2048;

#[contract]
impl ContractInterface for LetterContract {
    fn validate_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        _related: RelatedContracts<'static>,
    ) -> Result<ValidateResult, ContractError> {
        decode_state(state.as_ref()).map(|_| ValidateResult::Valid)
    }

    fn update_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
        data: Vec<UpdateData<'static>>,
    ) -> Result<UpdateModification<'static>, ContractError> {
        let mut current = decode_state(state.as_ref()).unwrap_or_default();
        for update in data {
            let bytes: Vec<u8> = match update {
                UpdateData::State(s) => s.as_ref().to_vec(),
                UpdateData::Delta(d) => d.as_ref().to_vec(),
                _ => continue,
            };
            let Ok(incoming) = bincode::deserialize::<StateData>(&bytes) else {
                continue;
            };
            for (pubkey, record) in incoming.peers {
                let Some(sig) = incoming.sigs.get(&pubkey) else {
                    continue;
                };
                if !verify_peer_sig(&pubkey, &record, sig) {
                    continue;
                }
                if current.peers.contains_key(&pubkey) {
                    continue;
                }
                current.peers.insert(pubkey, record);
                current.sigs.insert(pubkey, sig.clone());
            }
            for (seq, entry) in incoming.chain {
                if current.chain.len() >= MAX_CHAIN && !current.chain.contains_key(&seq) {
                    continue;
                }
                if !verify_chain_sig(&entry, seq) {
                    continue;
                }
                if let Some(existing) = current.chain.get(&seq) {
                    if existing == &entry {
                        continue;
                    }
                    continue;
                }
                if seq > 0 {
                    let Some(prev_entry) = current.chain.get(&(seq - 1)) else {
                        if seq != current.chain.len() as u64 {
                            let max_seq = current.chain.keys().copied().max().unwrap_or(0);
                            if seq > max_seq + 1 {
                                continue;
                            }
                        }
                        if current.chain.is_empty() && entry.prev != 0 {
                            continue;
                        }
                        if !current.chain.is_empty() {
                            continue;
                        }
                        current.chain.insert(seq, entry);
                        continue;
                    };
                    if prev_entry.next != entry.prev {
                        continue;
                    }
                } else if entry.prev != 0 {
                    continue;
                }
                current.chain.insert(seq, entry);
            }
        }
        Ok(UpdateModification::valid(encode_state(&current)))
    }

    fn summarize_state(
        _parameters: Parameters<'static>,
        state: State<'static>,
    ) -> Result<StateSummary<'static>, ContractError> {
        let data = decode_state(state.as_ref())?;
        let summary = bincode::serialize(&data).map_err(|e| ContractError::Deser(e.to_string()))?;
        Ok(StateSummary::from(summary))
    }

    fn get_state_delta(
        _parameters: Parameters<'static>,
        state: State<'static>,
        summary: StateSummary<'static>,
    ) -> Result<StateDelta<'static>, ContractError> {
        let ours = decode_state(state.as_ref())?;
        let theirs: StateData = match bincode::deserialize(summary.as_ref()) {
            Ok(s) => s,
            Err(_) => return Ok(StateDelta::from(state.as_ref().to_vec())),
        };
        let mut delta = StateData::default();
        for (k, v) in &ours.peers {
            if !theirs.peers.contains_key(k) {
                delta.peers.insert(*k, v.clone());
                if let Some(sig) = ours.sigs.get(k) {
                    delta.sigs.insert(*k, sig.clone());
                }
            }
        }
        for (seq, entry) in &ours.chain {
            if !theirs.chain.contains_key(seq) {
                delta.chain.insert(*seq, entry.clone());
            }
        }
        if delta.peers.is_empty() && delta.chain.is_empty() {
            return Ok(StateDelta::from(Vec::new()));
        }
        Ok(StateDelta::from(
            bincode::serialize(&delta).map_err(|e| ContractError::Deser(e.to_string()))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn key_for(tag: u8) -> (SigningKey, Pubkey) {
        let mut seed = [0u8; 32];
        seed[0] = tag;
        seed[1] = 0xAB;
        let sk = SigningKey::from_bytes(&seed);
        let pk = *VerifyingKey::from(&sk).as_bytes();
        (sk, pk)
    }

    fn peer_record_with_sig(tag: u8) -> (Pubkey, PeerRecord, Vec<u8>) {
        let (sk, pk) = key_for(tag);
        let rec = PeerRecord {
            peer_id: vec![tag, 1, 2],
            addrs: vec![format!("/ip4/127.0.0.1/tcp/{}", 4000 + u16::from(tag))],
        };
        let msg = bincode::serialize(&(&pk, &rec.peer_id, &rec.addrs)).unwrap();
        let sig = sk.sign(&msg).to_bytes().to_vec();
        (pk, rec, sig)
    }

    fn chain_entry(seq: u64, tag: u8, prev: u8, next: u8) -> (u64, ChainEntry) {
        let (sk, pk) = key_for(tag);
        let msg = bincode::serialize(&(seq, pk, prev, next)).unwrap();
        let sig = sk.sign(&msg).to_bytes().to_vec();
        (
            seq,
            ChainEntry {
                author: pk,
                prev,
                next,
                sig,
            },
        )
    }

    fn state_with(peers: Vec<(Pubkey, PeerRecord, Vec<u8>)>, chain: Vec<(u64, ChainEntry)>) -> State<'static> {
        let mut data = StateData::default();
        for (pk, rec, sig) in peers {
            data.peers.insert(pk, rec);
            data.sigs.insert(pk, sig);
        }
        for (seq, e) in chain {
            data.chain.insert(seq, e);
        }
        encode_state(&data)
    }

    fn params() -> Parameters<'static> {
        Parameters::from(bincode::serialize(&"test-lobby".to_string()).unwrap())
    }

    #[test]
    fn test_usage() {
        let (pk, rec, sig) = peer_record_with_sig(1);
        let (seq, entry) = chain_entry(0, 1, 0, b'a');
        let state = state_with(vec![(pk, rec, sig)], vec![(seq, entry)]);
        let related = RelatedContracts::default();
        assert!(LetterContract::validate_state(params(), state.clone(), related).is_ok());
        let summary = LetterContract::summarize_state(params(), state.clone()).unwrap();
        let _delta = LetterContract::get_state_delta(params(), state, summary).unwrap();
    }

    #[test]
    fn gossip_continuity_enforced() {
        let empty = StateData::default();
        let empty_state = encode_state(&empty);
        let (pk1, rec1, sig1) = peer_record_with_sig(1);
        let (s0, e0) = chain_entry(0, 1, 0, b'x');
        let (s1, e1) = chain_entry(1, 2, b'x', b'y');
        let bad = {
            let (sk, pk) = key_for(2);
            let msg = bincode::serialize(&(1u64, pk, b'Z', b'y')).unwrap();
            let sig = sk.sign(&msg).to_bytes().to_vec();
            ChainEntry { author: pk, prev: b'Z', next: b'y', sig }
        };
        let first = {
            let mut d = StateData::default();
            d.peers.insert(pk1, rec1.clone());
            d.sigs.insert(pk1, sig1.clone());
            d.chain.insert(s0, e0.clone());
            State::from(bincode::serialize(&d).unwrap())
        };
        let res = LetterContract::update_state(params(), empty_state, vec![UpdateData::State(first)]).unwrap();
        let st = res.unwrap_valid();
        let ok_second = {
            let mut d = StateData::default();
            d.chain.insert(s1, e1.clone());
            State::from(bincode::serialize(&d).unwrap())
        };
        let res = LetterContract::update_state(params(), st.clone(), vec![UpdateData::State(ok_second)]).unwrap();
        let st2 = res.unwrap_valid();
        let decoded = bincode::deserialize::<StateData>(st2.as_ref()).unwrap();
        assert_eq!(decoded.chain.len(), 2);
        let bad_second = {
            let mut d = StateData::default();
            d.chain.insert(1, bad);
            State::from(bincode::serialize(&d).unwrap())
        };
        let res = LetterContract::update_state(params(), st2.clone(), vec![UpdateData::State(bad_second)]).unwrap();
        let st3 = res.unwrap_valid();
        let decoded = bincode::deserialize::<StateData>(st3.as_ref()).unwrap();
        assert_eq!(decoded.chain.len(), 2, "bad continuity should be rejected");
    }

    #[test]
    fn fork_first_writer_wins() {
        let (pk1, rec1, sig1) = peer_record_with_sig(1);
        let (s0, e0) = chain_entry(0, 1, 0, b'a');
        let base = state_with(vec![(pk1, rec1, sig1)], vec![(s0, e0)]);
        let (s1a, e1a) = chain_entry(1, 2, b'a', b'b');
        let (s1b, _) = chain_entry(1, 2, b'a', b'c');
        let mut d_a = StateData::default();
        d_a.chain.insert(s1a, e1a.clone());
        let mut d_b = StateData::default();
        let (sk, pk) = key_for(2);
        let msg = bincode::serialize(&(1u64, pk, b'a', b'c')).unwrap();
        let sig = sk.sign(&msg).to_bytes().to_vec();
        d_b.chain.insert(s1b, ChainEntry { author: pk, prev: b'a', next: b'c', sig });
        let res_a = LetterContract::update_state(
            params(),
            base.clone(),
            vec![UpdateData::State(State::from(bincode::serialize(&d_a).unwrap()))],
        )
        .unwrap()
        .unwrap_valid();
        let res_b = LetterContract::update_state(
            params(),
            res_a.clone(),
            vec![UpdateData::State(State::from(bincode::serialize(&d_b).unwrap()))],
        )
        .unwrap()
        .unwrap_valid();
        let decoded = bincode::deserialize::<StateData>(res_b.as_ref()).unwrap();
        assert_eq!(decoded.chain.get(&1).unwrap().next, b'b', "first writer wins");
    }
}
