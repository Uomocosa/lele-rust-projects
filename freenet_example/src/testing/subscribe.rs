use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::FreenetClient;

pub async fn subscribe(client: &mut FreenetClient, key: ContractKey) -> u64 {
    let get_req = ContractRequest::Get {
        key: *key.id(),
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(get_req)).await.unwrap();
    loop {
        match client.recv_response().await.unwrap() {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                return bincode::deserialize(state.as_ref()).unwrap();
            }
            other => panic!("unexpected subscribe response: {other:?}"),
        }
    }
}
