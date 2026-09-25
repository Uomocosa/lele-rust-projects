use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use super::super::error::Error;
use super::catalogue::RoomCatalogue;
use super::index_client::IndexClient;
use crate::discovery;

pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
) -> Result<IndexClient, Error> {
    let mut client = discovery::link::Client::connect(host, port).await?;
    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    match recv_after_get(&mut client, instance_id).await {
        Ok(()) => {}
        Err(Error::ContractNotFound) => {
            let put = ContractRequest::Put {
                contract: container.clone(),
                state: WrappedState::new(bincode::serialize(&RoomCatalogue::new())?),
                related_contracts: RelatedContracts::default(),
                subscribe: true,
                blocking_subscribe: false,
            };
            client.send(&ClientRequest::ContractOp(put))?;
            match client.recv_response().await? {
                HostResponse::ContractResponse(
                    ContractResponse::PutResponse { key }
                    | ContractResponse::SubscribeResponse { key, .. }
                    | ContractResponse::UpdateResponse { key, .. },
                ) => {
                    info!(target: "room_lobby", key = %key, "discovery: board contract deployed");
                }
                other => return Err(Error::UnexpectedResponse(format!("{other:?}"))),
            }
            recv_after_get(&mut client, instance_id).await?;
        }
        Err(e) => return Err(e),
    }
    Ok(IndexClient {
        client,
        contract_key,
    })
}

// needed helper: issues a blocking subscribe-get and ignores the first state
async fn recv_after_get(
    client: &mut discovery::link::Client,
    instance_id: ContractInstanceId,
) -> Result<(), Error> {
    let get = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(&ClientRequest::ContractOp(get))?;
    loop {
        match client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { .. }) => return Ok(()),
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(Error::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => return Err(Error::UnexpectedResponse(format!("{other:?}"))),
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
