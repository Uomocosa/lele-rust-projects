use crate::clicker_client;
use crate::clicker_error;
use crate::freenet_client;
use std::collections::BTreeMap;
use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::Role;
use crate::recv_after_get;
use crate::recv_response;
use clicker_error::ClickerError;
use clicker_error::ClickerError as Ce;
use freenet_client::FreenetClient;

/// # Errors
/// Returns `ClickerError` if the connection or contract get/put fails.
pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    role: Role,
    tag: u64,
) -> Result<clicker_client::ClickerClient, ClickerError> {
    let mut client = FreenetClient::connect(host, port).await?;

    let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let params = Parameters::from(params.to_vec());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));

    let (key, slots) = match role {
        Role::Publish => {
            let result = recv_after_get(&mut client, instance_id).await;
            if let Ok(r) = result {
                let (key, slots) = r;
                (key, slots)
            } else {
                let initial = BTreeMap::from([(tag, 0u64)]);
                let put_req = ContractRequest::Put {
                    contract: container.clone(),
                    state: WrappedState::new(bincode::serialize(&initial)?),
                    related_contracts: RelatedContracts::default(),
                    subscribe: true,
                    blocking_subscribe: false,
                };
                client.send(ClientRequest::ContractOp(put_req)).await?;
                match recv_response(&mut client).await? {
                    HostResponse::ContractResponse(
                        ContractResponse::PutResponse { key }
                        | ContractResponse::SubscribeResponse { key, .. }
                        | ContractResponse::UpdateResponse { key, .. },
                    ) => {
                        info!(target: "freenet_example", key = %key, "contract deployed");
                    }
                    other => return Err(Ce::UnexpectedResponse(format!("{other:?}"))),
                }
                let (key, slots) = recv_after_get(&mut client, instance_id).await?;
                (key, slots)
            }
        }
        Role::Subscribe => {
            let r = loop {
                if let Ok(r) = recv_after_get(&mut client, instance_id).await {
                    break r;
                }
                info!(
                    target: "freenet_example",
                    %instance_id,
                    "contract not found, retrying in 1s"
                );
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            };
            (r.0, r.1)
        }
    };

    let foreign_sum: u64 = slots
        .iter()
        .filter(|(t, _)| **t != tag)
        .map(|(_, v)| v)
        .sum();
    Ok(clicker_client::ClickerClient {
        client,
        contract_key: key,
        slots,
        tag,
        foreign_seen: (foreign_sum > 0).then(std::time::Instant::now),
        foreign_sum,
        last_bridge: None,
        contract: container,
    })
}

#[cfg(all(test, feature = "dev"))]
mod tests {
    use crate::testing::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        let wasm = load_wasm();
        let mut client = connect(node.port).await.unwrap();
        let key = deploy(&mut client, &wasm).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 0);
        update_count(&mut client, key, 0, 42).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 42);
        update_count(&mut client, key, 0, 99).await.unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 99);
    }
}
