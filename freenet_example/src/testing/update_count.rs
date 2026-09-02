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

/// # Errors
/// Returns `ClientError` if the update request fails or the response is unexpected.
pub async fn update_count(
    client: &mut FreenetClient,
    key: ContractKey,
    tag: u64,
    count: u64,
) -> Result<(), ClientError> {
    let mut seed = [0u8; 32];
    seed[0..8].copy_from_slice(&tag.to_le_bytes());
    let sk = SigningKey::from_bytes(&seed);
    let msg = bincode::serialize(&(tag, count))?;
    let sig = sk.sign(&msg);
    let mut sigs = BTreeMap::new();
    sigs.insert(tag, sig.to_bytes().to_vec());
    let signed = SignedSlots {
        slots: BTreeMap::from([(tag, count)]),
        sigs,
    };
    let state = State::from(bincode::serialize(&signed)?);
    let req = ContractRequest::Update {
        key,
        data: UpdateData::State(state),
    };
    client.send(ClientRequest::ContractOp(req)).await?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => Ok(()),
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}

// no test_usage necessary — exercised via integration tests
