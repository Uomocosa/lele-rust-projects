use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, HostResponse};
use freenet_stdlib::prelude::{
    ContractCode, ContractContainer, ContractWasmAPIVersion, Parameters, RelatedContracts,
    WrappedContract, WrappedState,
};

use crate::discovery;
use crate::discovery_update_data_bytes;
use crate::freenet_client;

/// # Errors
/// Returns error if connection fails.
///
/// # Panics
/// May panic if serialization fails.
pub async fn connect(
    host: &str,
    port: u16,
    wasm: &[u8],
    lobby: &str,
) -> Result<discovery::Discovery, String> {
    let mut client = freenet_client::FreenetClient::connect(host, port).await?;
    let serialized = bincode::serialize(&lobby.to_string()).unwrap_or_default();
    let params = Parameters::from(serialized);
    let code = Arc::new(ContractCode::from(wasm.to_vec()));
    let wrapped = WrappedContract::new(code, params);
    let key = wrapped.key;
    let instance = *key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    let get_req = ClientRequest::ContractOp(ContractRequest::Get {
        key: instance,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: false,
    });
    client.send(get_req).await?;
    let mut peers = BTreeMap::new();
    let mut chain = BTreeMap::new();
    let timeout = Duration::from_secs(3);
    if let Some(res) = client.recv_with_timeout(timeout).await
        && let Ok(HostResponse::ContractResponse(
            freenet_stdlib::client_api::ContractResponse::GetResponse { state, .. },
        )) = res
        && let Ok(data) = bincode::deserialize::<discovery::StateData>(state.as_ref())
    {
        peers = data.peers;
        chain = data.chain;
    }
    while let Some(Ok(HostResponse::ContractResponse(
        freenet_stdlib::client_api::ContractResponse::UpdateNotification { update, .. },
    ))) = client.recv_with_timeout(Duration::from_millis(200)).await
    {
        if let Some(bytes) = discovery_update_data_bytes::update_data_bytes(&update)
            && let Ok(data) = bincode::deserialize::<discovery::StateData>(&bytes)
        {
            for (k, v) in data.peers {
                peers.entry(k).or_insert(v);
            }
            for (seq, e) in data.chain {
                chain.entry(seq).or_insert(e);
            }
        }
    }
    if peers.is_empty() && chain.is_empty() {
        let state = discovery::StateData::default();
        let serialized_state = bincode::serialize(&state).unwrap_or_default();
        let put_req = ClientRequest::ContractOp(ContractRequest::Put {
            contract: container.clone(),
            state: WrappedState::new(serialized_state),
            related_contracts: RelatedContracts::default(),
            subscribe: true,
            blocking_subscribe: false,
        });
        let _ = client.send(put_req).await;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(discovery::Discovery {
        client,
        key,
        lobby: lobby.to_string(),
        peers,
        chain,
        last_bridge: None,
        foreign_len: 0,
        contract_wasm: wasm.to_vec(),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(connect);
    }
}
