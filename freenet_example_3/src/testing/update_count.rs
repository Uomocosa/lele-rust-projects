use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClientError;
use crate::FreenetClient;

/// # Errors
/// Returns `ClientError` if the update request fails or the response is unexpected.
pub async fn update_count(
    client: &mut FreenetClient,
    key: ContractKey,
    tag: u64,
    count: u64,
) -> Result<(), ClientError> {
    let slots = BTreeMap::from([(tag, count)]);
    let state = State::from(bincode::serialize(&slots)?);
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
