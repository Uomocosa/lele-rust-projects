use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::structs::client_error::ClientError;
use crate::structs::freenet_client::FreenetClient;

pub async fn subscribe(client: &mut FreenetClient, key: ContractKey) -> Result<u64, ClientError> {
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
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}
