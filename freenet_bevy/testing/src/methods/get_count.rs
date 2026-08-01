use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::ContractKey;

pub async fn get_count(
    client: &mut freenet_bevy::freenet::FreenetClient,
    key: ContractKey,
) -> Result<u64, freenet_bevy::freenet::FreenetConnectionError> {
    let get_req = ContractRequest::Get {
        key: *key.id(),
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;
    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
            Ok(bincode::deserialize(state.as_ref())?)
        }
        other => Err(freenet_bevy::freenet::FreenetConnectionError::UnexpectedResponse(format!(
            "{other:?}"
        ))),
    }
}
