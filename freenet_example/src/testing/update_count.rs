use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClientError;
use crate::FreenetClient;

pub async fn update_count(
    client: &mut FreenetClient,
    key: ContractKey,
    count: u64,
) -> Result<(), ClientError> {
    let state = State::from(bincode::serialize(&count)?);
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
