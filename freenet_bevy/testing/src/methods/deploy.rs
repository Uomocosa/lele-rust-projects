use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

pub async fn deploy(
    client: &mut freenet_bevy::freenet::FreenetClient,
    wasm: &[u8],
) -> Result<ContractKey, freenet_bevy::freenet::FreenetConnectionError> {
    let code = Arc::new(ContractCode::from(wasm.to_vec()));
    let params = Parameters::from(Vec::new());
    let wrapped = WrappedContract::new(code, params);
    let key = wrapped.key;
    let instance_id = *key.id();

    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;

    match client.recv_response().await? {
        HostResponse::ContractResponse(ContractResponse::GetResponse { key, .. }) => {
            return Ok(key);
        }
        HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {}
        other => {
            return Err(freenet_bevy::freenet::FreenetConnectionError::UnexpectedResponse(format!(
                "{other:?}"
            )))
        }
    }

    let put_req = ContractRequest::Put {
        contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
        state: WrappedState::new(bincode::serialize(&0u64)?),
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(put_req)).await?;

    match client.recv_response().await? {
        HostResponse::ContractResponse(
            ContractResponse::PutResponse { key }
            | ContractResponse::SubscribeResponse { key, .. },
        ) => Ok(key),
        other => Err(freenet_bevy::freenet::FreenetConnectionError::UnexpectedResponse(format!(
            "{other:?}"
        ))),
    }
}
