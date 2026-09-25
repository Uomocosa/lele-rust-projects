use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;
use discovery::Error;
use discovery::lobby_rooms::{IndexClient, RoomCatalogue};

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
    let slots = match recv_board(&mut client, instance_id).await {
        Ok(board) => board,
        Err(Error::ContractNotFound) => {
            let put = ContractRequest::Put {
                contract: container,
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
            recv_board(&mut client, instance_id).await?
        }
        Err(e) => return Err(e),
    };
    Ok(IndexClient {
        client,
        contract_key,
        slots,
    })
}

// needed helper: issues a blocking subscribe-get and returns the current board
async fn recv_board(
    client: &mut discovery::link::Client,
    instance_id: ContractInstanceId,
) -> Result<RoomCatalogue, Error> {
    let get = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(&ClientRequest::ContractOp(get))?;
    loop {
        match client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                return Ok(bincode::deserialize(state.as_ref()).unwrap_or_default());
            }
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
