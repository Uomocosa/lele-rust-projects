use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::ContractKey;

use crate::structs::client_error::ClientError;
use crate::structs::freenet_client::FreenetClient;

pub async fn get_count(client: &mut FreenetClient, key: ContractKey) -> Result<u64, ClientError> {
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
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}
