use crate::client_error;
use crate::freenet_client;
use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::recv_response;
use client_error::ClientError;

/// # Errors
/// Returns `ClientError` if the get fails or the response is unexpected.
pub async fn recv_after_get(
    client: &mut freenet_client::FreenetClient,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, BTreeMap<u64, u64>), ClientError> {
    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;
    loop {
        match recv_response(client).await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse {
                key, state, ..
            }) => {
                let slots = bincode::deserialize(state.as_ref()).unwrap_or_default();
                return Ok((key, slots));
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(ClientError::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => return Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}

// no test_usage necessary — exercised via integration tests
