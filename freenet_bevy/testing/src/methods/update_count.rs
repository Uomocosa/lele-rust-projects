use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

pub async fn update_count(
    client: &mut freenet_bevy::FreenetClient,
    key: ContractKey,
    count: u64,
) -> Result<(), freenet_bevy::ClientError> {
    let update_req = ContractRequest::Update {
        key,
        data: UpdateData::State(State::from(bincode::serialize(&count)?)),
    };
    client.send(ClientRequest::ContractOp(update_req)).await?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => Ok(()),
        other => Err(freenet_bevy::ClientError::UnexpectedResponse(format!(
            "{other:?}"
        ))),
    }
}
