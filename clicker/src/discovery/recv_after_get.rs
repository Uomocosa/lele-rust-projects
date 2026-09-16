use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::discovery;

/// # Errors
/// Returns `Error` if the get fails, times out, or the response is unexpected.
pub async fn recv_after_get(
    client: &mut discovery::Client,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, discovery::RosterState), discovery::Error> {
    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(&ClientRequest::ContractOp(get_req))?;
    let wait = Duration::from_secs(discovery::CONNECT_TIMEOUT_SECS);
    loop {
        let response = tokio::time::timeout(wait, client.recv_response())
            .await
            .map_err(|_| discovery::Error::ResponseTimeout)?;
        match response? {
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
