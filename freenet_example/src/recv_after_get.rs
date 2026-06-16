use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClientError;
use crate::recv_response;

pub(crate) async fn recv_after_get(
    client: &mut crate::FreenetClient,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, u64), ClientError> {
    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;
    match recv_response(client).await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { key, state, .. }) => {
            let count = bincode::deserialize(state.as_ref()).unwrap_or(0);
            Ok((key, count))
        }
        HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
            Err(ClientError::ContractNotFound)
        }
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}
