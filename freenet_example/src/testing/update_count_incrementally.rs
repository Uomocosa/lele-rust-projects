use std::collections::BTreeMap;

use ed25519_dalek::{Signer, SigningKey};
use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ClientError;
use crate::FreenetClient;

#[derive(Serialize, Deserialize)]
struct SignedSlots {
    slots: BTreeMap<u64, u64>,
    sigs: BTreeMap<u64, Vec<u8>>,
}

async fn get_slot(
    client: &mut FreenetClient,
    key: ContractKey,
    tag: u64,
) -> Result<u64, ClientError> {
    let get_req = ContractRequest::Get {
        key: *key.id(),
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
            let slots: BTreeMap<u64, u64> = bincode::deserialize(state.as_ref())?;
            Ok(slots.get(&tag).copied().unwrap_or(0))
        }
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}

/// # Errors
/// Returns `ClientError` if the get or any incremental update fails.
pub async fn update_count_incrementally(
    client: &mut FreenetClient,
    key: ContractKey,
    tag: u64,
    target: u64,
) -> Result<(), ClientError> {
    let cur = get_slot(client, key, tag).await?;
    if target <= cur {
        return Ok(());
    }
    for v in (cur.saturating_add(1))..=target {
        let mut seed = [0u8; 32];
        seed[0..8].copy_from_slice(&tag.to_le_bytes());
        let sk = SigningKey::from_bytes(&seed);
        let msg =
            bincode::serialize(&(tag, v)).map_err(|e| ClientError::Serialization(e.to_string()))?;
        let sig = sk.sign(&msg);
        let mut sigs = BTreeMap::new();
        sigs.insert(tag, sig.to_bytes().to_vec());
        let signed = SignedSlots {
            slots: BTreeMap::from([(tag, v)]),
            sigs,
        };
        let state = State::from(
            bincode::serialize(&signed).map_err(|e| ClientError::Serialization(e.to_string()))?,
        );
        let req = ContractRequest::Update {
            key,
            data: UpdateData::State(state),
        };
        client.send(ClientRequest::ContractOp(req)).await?;
        match client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
            other => return Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
        }
    }
    Ok(())
}

// no test_usage necessary — exercised via integration tests
