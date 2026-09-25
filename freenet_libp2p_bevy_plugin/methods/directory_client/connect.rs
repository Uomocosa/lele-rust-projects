use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
) -> Result<discovery::link::DirectoryClient, discovery::Error> {
    let mut client = discovery::link::Client::connect(host, port).await?;
    let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let params = Parameters::from(params.to_vec());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    let initial = discovery::directory::DirectoryState::new();
    let (key, slots) = match recv_after_get_directory(&mut client, instance_id).await {
        Ok(found) => found,
        Err(discovery::Error::ContractNotFound) => {
            let put_req = ContractRequest::Put {
                contract: container.clone(),
                state: WrappedState::new(bincode::serialize(&initial)?),
                related_contracts: RelatedContracts::default(),
                subscribe: true,
                blocking_subscribe: false,
            };
            client.send(&ClientRequest::ContractOp(put_req))?;
            match client.recv_response().await? {
                HostResponse::ContractResponse(
                    ContractResponse::PutResponse { key }
                    | ContractResponse::SubscribeResponse { key, .. }
                    | ContractResponse::UpdateResponse { key, .. },
                ) => {
                    info!(target: "room_lobby", key = %key, "directory contract deployed");
                }
                other => {
                    return Err(discovery::Error::UnexpectedResponse(format!("{other:?}")));
                }
            }
            recv_after_get_directory(&mut client, instance_id).await?
        }
        Err(e) => return Err(e),
    };
    Ok(discovery::link::DirectoryClient {
        client,
        contract_key: key,
        contract: container,
        slots,
        last_bridge: None,
    })
}

// needed helper: issues a blocking subscribe-get and reads the first state
async fn recv_after_get_directory(
    client: &mut discovery::link::Client,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, discovery::directory::DirectoryState), discovery::Error> {
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
            other => return Err(discovery::Error::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::connect;

    #[tokio::test]
    async fn test_usage() {
        assert!(connect("127.0.0.1", 1, &[], &[]).await.is_err());
    }
}
