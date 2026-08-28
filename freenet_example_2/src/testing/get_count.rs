use std::collections::BTreeMap;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::ClientError;
use crate::FreenetClient;

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
            let slots: BTreeMap<u64, u64> = bincode::deserialize(state.as_ref())?;
            Ok(slots.values().sum())
        }
        other => Err(ClientError::UnexpectedResponse(format!("{other:?}"))),
    }
}

// no test_usage necessary — exercised via integration tests
