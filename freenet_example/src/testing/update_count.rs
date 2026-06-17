use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::FreenetClient;

pub async fn update_count(client: &mut FreenetClient, key: ContractKey, count: u64) {
    let state = State::from(bincode::serialize(&count).unwrap());
    let req = ContractRequest::Update {
        key,
        data: UpdateData::State(state),
    };
    client.send(ClientRequest::ContractOp(req)).await.unwrap();
    match client.recv_response().await.unwrap() {
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => {}
        other => panic!("unexpected UPDATE response: {other:?}"),
    }
}
