use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClientError;
use crate::FreenetClient;

use super::update_count;

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
        update_count(client, key, tag, v).await?;
    }
    Ok(())
}

// no test_usage necessary — exercised via integration tests
