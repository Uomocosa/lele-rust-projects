use crate::global_counter_client;
use crate::global_counter_client_method;
use crate::global_counter_error;
use std::collections::BTreeMap;
use std::time::Duration;

use ed25519_dalek::{Signer, SigningKey};
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

use global_counter_error::GlobalCounterError as Ce;

#[derive(Serialize, Deserialize)]
struct SignedSlots {
    slots: BTreeMap<u64, u64>,
    sigs: BTreeMap<u64, Vec<u8>>,
}

// needed helper:
fn absorb_slots(slots: &mut BTreeMap<u64, u64>, bytes: &[u8]) {
    if let Ok(incoming) = bincode::deserialize::<BTreeMap<u64, u64>>(bytes) {
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

/// # Errors
/// Returns `GlobalCounterError` if serialization fails or the update response is unexpected.
/// # Panics
/// Panics only if serialization fails due to internal error, which is propagated as `GlobalCounterError`.
pub async fn tick(
    client: &mut global_counter_client::GlobalCounterClient,
) -> Result<u64, global_counter_error::GlobalCounterError> {
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
        .get(&client.tag)
        .copied()
        .unwrap_or(0)
        .wrapping_add(1);
    client.slots.insert(client.tag, own);
    let mut sigs = BTreeMap::new();
    let sk = signing_key_for_tag(client.tag);
    let msg = bincode::serialize(&(client.tag, own))?;
    let sig = sk.sign(&msg);
    sigs.insert(client.tag, sig.to_bytes().to_vec());
    let signed = SignedSlots {
        slots: BTreeMap::from([(client.tag, own)]),
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
    use std::collections::BTreeMap;

    #[test]
    fn test_usage() {
        let mut slots = BTreeMap::new();
        let bytes = bincode::serialize(&BTreeMap::from([(0u64, 5u64), (2, 7)])).unwrap();
        absorb_slots(&mut slots, &bytes);
        assert_eq!(slots.get(&0), Some(&5));
        assert_eq!(slots.values().sum::<u64>(), 12);
    }
}
