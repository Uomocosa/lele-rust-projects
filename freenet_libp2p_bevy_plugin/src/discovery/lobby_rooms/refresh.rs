use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};

use super::super::error::Error;
use super::catalogue::RoomCatalogue;
use super::index_client::IndexClient;
use super::merge_board::merge_board;

pub async fn refresh(client: &mut IndexClient) -> Result<RoomCatalogue, Error> {
    let instance_id = *client.contract_key.id();
    let get = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.client.send(&ClientRequest::ContractOp(get))?;
    loop {
        match client.client.recv_response().await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                let board: RoomCatalogue = bincode::deserialize(state.as_ref()).unwrap_or_default();
                client.slots = merge_board(client.slots.clone(), board);
                return Ok(client.slots.clone());
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(Error::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => return Err(Error::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}

// no test_usage necessary
