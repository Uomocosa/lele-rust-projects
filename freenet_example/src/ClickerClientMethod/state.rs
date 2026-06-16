use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClickerError as Ce;

pub async fn state(client: &mut crate::ClickerClient) -> Result<u64, crate::ClickerError> {
    let get_req = ContractRequest::Get {
        key: *client.contract_key.id(),
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client
        .client
        .send(ClientRequest::ContractOp(get_req))
        .await?;
    match client.client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
            let count = bincode::deserialize(state.as_ref()).unwrap_or(0);
            client.count = count;
            Ok(count)
        }
        other => Err(Ce::UnexpectedResponse(format!("{other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {}
}
