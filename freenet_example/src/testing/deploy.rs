use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::FreenetClient;

pub async fn deploy(client: &mut FreenetClient, wasm: &[u8]) -> ContractKey {
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
    client.send(ClientRequest::ContractOp(get_req)).await.unwrap();

    loop {
        match client.recv_response().await.unwrap() {
            HostResponse::ContractResponse(ContractResponse::GetResponse { key, .. }) => return key,
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => break,
            other => panic!("unexpected response to initial GET: {other:?}"),
        }
    }

    let put_req = ContractRequest::Put {
        contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
        state: WrappedState::new(bincode::serialize(&0u64).unwrap()),
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(ClientRequest::ContractOp(put_req)).await.unwrap();

    match client.recv_response().await.unwrap() {
        HostResponse::ContractResponse(
            ContractResponse::PutResponse { key }
            | ContractResponse::SubscribeResponse { key, .. },
        ) => key,
        other => panic!("unexpected response to PUT: {other:?}"),
    }
}
