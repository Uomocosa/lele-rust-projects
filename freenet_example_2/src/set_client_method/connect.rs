use crate::clicker_error;
use crate::freenet_client;
use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::set_client;
use clicker_error::ClickerError as Ce;
use freenet_client::FreenetClient;

// needed helper:
async fn recv_set(client: &mut FreenetClient) -> Result<Option<BTreeSet<u64>>, Ce> {
    loop {
        match client.recv_response_timeout(Duration::from_secs(60)).await {
            Some(Ok(HostResponse::ContractResponse(ContractResponse::GetResponse {
                state,
                ..
            }))) => {
                let set = bincode::deserialize::<BTreeSet<u64>>(state.as_ref())
                    .map_err(Ce::Serialization)?;
                return Ok(Some(set));
            }
            Some(Ok(HostResponse::ContractResponse(ContractResponse::NotFound { .. }))) => {
                return Ok(None);
            }
            Some(Ok(HostResponse::ContractResponse(ContractResponse::SubscribeResponse {
                ..
            }))) => {
                continue;
            }
            Some(Ok(HostResponse::ContractResponse(ContractResponse::PutResponse { .. }))) => {
                continue;
            }
            Some(Ok(HostResponse::ContractResponse(ContractResponse::UpdateResponse {
                ..
            }))) => {
                continue;
            }
            Some(Ok(other)) => return Err(Ce::UnexpectedResponse(format!("{other:?}"))),
            Some(Err(e)) => return Err(Ce::Client(e)),
            None => return Err(Ce::Timeout),
        }
    }
}

pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    tag: u64,
) -> Result<set_client::SetClient, Ce> {
    let mut client = FreenetClient::connect(host, port).await?;

    let code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let wrapped = WrappedContract::new(code, Parameters::from(params.to_vec()));
    let key = wrapped.key;
    let instance_id = *key.id();

    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: false,
    };
    client.send(ClientRequest::ContractOp(get_req)).await?;

    let set = match recv_set(&mut client).await? {
        Some(set) => set,
        None => {
            let empty = BTreeSet::<u64>::new();
            let put_req = ContractRequest::Put {
                contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
                state: WrappedState::new(bincode::serialize(&empty)?),
                related_contracts: RelatedContracts::default(),
                subscribe: true,
                blocking_subscribe: false,
            };
            client.send(ClientRequest::ContractOp(put_req)).await?;
            let reget = ContractRequest::Get {
                key: instance_id,
                return_contract_code: false,
                subscribe: true,
                blocking_subscribe: false,
            };
            client.send(ClientRequest::ContractOp(reget)).await?;
            recv_set(&mut client).await?.ok_or(Ce::ContractNotFound)?
        }
    };

    Ok(set_client::SetClient {
        client,
        contract_key: key,
        set,
        tag,
        seq: 0,
    })
}

// no test_usage necessary — needs a live embedded node, exercised by e2e
