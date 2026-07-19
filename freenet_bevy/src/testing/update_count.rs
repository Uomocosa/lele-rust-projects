use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClientError;
use crate::FreenetClient;

pub async fn update_count(
    client: &mut FreenetClient,
    key: ContractKey,
    count: u64,
) -> Result<(), ClientError> {
    let update_req = ContractRequest::Update {
        key,
        data: UpdateData::State(State::from(bincode::serialize(&count)?)),
    };
    client.send(ClientRequest::ContractOp(update_req)).await?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => Ok(()),
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}
