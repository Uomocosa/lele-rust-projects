use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

pub async fn subscribe(
    client: &mut freenet_bevy::FreenetClient,
    key: ContractKey,
) -> Result<u64, freenet_bevy::ClientError> {
    let get_req = ContractRequest::Get {
        key: *key.id(),
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
            Ok(bincode::deserialize(state.as_ref())?)
        }
        other => Err(freenet_bevy::ClientError::UnexpectedResponse(format!(
            "{other:?}"
        ))),
    }
}
