use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::discovery;

/// # Errors
/// Returns `Error` if the get fails or the response is unexpected.
pub async fn recv_after_get_directory(
    client: &mut discovery::Client,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, discovery::DirectoryState), discovery::Error> {
    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(&ClientRequest::ContractOp(get_req))?;
    loop {
        match client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse {
                key, state, ..
            }) => {
                let slots = bincode::deserialize(state.as_ref()).unwrap_or_default();
                return Ok((key, slots));
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(discovery::Error::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => {
                return Err(discovery::Error::UnexpectedResponse(format!("{other:?}")));
            }
        }
    }
}
// no test_usage necessary
