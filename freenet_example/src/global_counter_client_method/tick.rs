use crate::global_counter_client;
use crate::global_counter_client_method;
use std::collections::BTreeMap;
use std::time::Duration;

use ed25519_dalek::{Signer, SigningKey};
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

use crate::global_counter_client::GlobalCounterClient;
use crate::global_counter_error::GlobalCounterError as Ce;
use crate::global_counter_error::GlobalCounterError;

#[derive(Serialize, Deserialize)]
struct SignedSlots {
    slots: BTreeMap<global_counter_client::Pubkey, u64>,
    sigs: BTreeMap<global_counter_client::Pubkey, Vec<u8>>,
}

// needed helper:
fn absorb_slots(slots: &mut BTreeMap<global_counter_client::Pubkey, u64>, bytes: &[u8]) {
    if let Ok(incoming) =
        bincode::deserialize::<BTreeMap<global_counter_client::Pubkey, u64>>(bytes)
    {
        global_counter_client_method::merge_slots(slots, incoming);
        return;
    }
    if let Ok(signed) = bincode::deserialize::<SignedSlots>(bytes) {
        global_counter_client_method::merge_slots(slots, signed.slots);
    }
}

fn signing_key_for_tag(tag: u64) -> SigningKey {
    let mut seed = [0u8; 32];
    seed[0..8].copy_from_slice(&tag.to_le_bytes());
    SigningKey::from_bytes(&seed)
}

fn pubkey_for_tag(tag: u64) -> global_counter_client::Pubkey {
    let sk = signing_key_for_tag(tag);
    let vk = ed25519_dalek::VerifyingKey::from(&sk);
    *vk.as_bytes()
}

/// # Errors
/// Returns `GlobalCounterError` if serialization fails or the update response is unexpected.
/// # Panics
/// Panics only if serialization fails due to internal error, which is propagated as `GlobalCounterError`.
pub async fn tick(client: &mut GlobalCounterClient) -> Result<u64, GlobalCounterError> {
    while let Some(result) = client.client.recv_timeout(Duration::from_millis(10)).await {
        if let Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update,
            ..
        })) = result
        {
            match update {
                UpdateData::State(s) => absorb_slots(&mut client.slots, s.as_ref()),
                UpdateData::Delta(d) => absorb_slots(&mut client.slots, d.as_ref()),
                _ => {}
            }
        }
    }

    let own = client
        .slots
        .get(&client.pubkey)
        .copied()
        .unwrap_or(0)
        .wrapping_add(1);
    client.slots.insert(client.pubkey, own);
    let mut sigs = BTreeMap::new();
    let sk = signing_key_for_tag(client.tag);
    let pk = pubkey_for_tag(client.tag);
    debug_assert_eq!(pk, client.pubkey);
    let msg = bincode::serialize(&(pk, own))?;
    let sig = sk.sign(&msg);
    sigs.insert(pk, sig.to_bytes().to_vec());
    let signed = SignedSlots {
        slots: BTreeMap::from([(pk, own)]),
        sigs,
    };
    let new_state = State::from(bincode::serialize(&signed)?);
    let update_req = ContractRequest::Update {
        key: client.contract_key,
        data: UpdateData::State(new_state),
    };
    client
        .client
        .send(ClientRequest::ContractOp(update_req))
        .await?;

    match client.client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
        other => return Err(Ce::UnexpectedResponse(format!("{other:?}"))),
    }

    Ok(client.slots.values().sum())
}

#[cfg(test)]
mod tests {
    use super::absorb_slots;
    use crate::global_counter_client;

    use std::collections::BTreeMap;

    fn pk(n: u8) -> global_counter_client::Pubkey {
        let mut p = [0u8; 32];
        p[0] = n;
        p
    }

    #[test]
    fn test_usage() {
        let mut slots = BTreeMap::new();
        let bytes = bincode::serialize(&BTreeMap::from([(pk(0), 5u64), (pk(2), 7)])).unwrap();
        absorb_slots(&mut slots, &bytes);
        assert_eq!(slots.get(&pk(0)), Some(&5));
        assert_eq!(slots.values().sum::<u64>(), 12);
    }
}
