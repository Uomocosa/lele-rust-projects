use crate::freenet_client;
use crate::global_counter_client;
use crate::global_counter_error;
use global_counter_client::Pubkey;
use std::collections::BTreeMap;
use std::sync::Arc;

use ed25519_dalek::{SigningKey, VerifyingKey};

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::Role;
use crate::recv_after_get;
use crate::recv_response;
use freenet_client::FreenetClient;
use global_counter_error::GlobalCounterError;
use global_counter_error::GlobalCounterError as Ce;

/// # Errors
/// Returns `GlobalCounterError` if the connection or contract get/put fails.
pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    role: Role,
    tag: u64,
) -> Result<global_counter_client::GlobalCounterClient, GlobalCounterError> {
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
                let pk = {
                    let mut seed = [0u8; 32];
                    seed[0..8].copy_from_slice(&tag.to_le_bytes());
                    let sk = SigningKey::from_bytes(&seed);
                    let vk = VerifyingKey::from(&sk);
                    *vk.as_bytes()
                };
                let initial: BTreeMap<Pubkey, u64> = BTreeMap::from([(pk, 0u64)]);
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

    let pubkey: Pubkey = {
        let mut seed = [0u8; 32];
        seed[0..8].copy_from_slice(&tag.to_le_bytes());
        let sk = SigningKey::from_bytes(&seed);
        let vk = VerifyingKey::from(&sk);
        *vk.as_bytes()
    };
    let foreign_sum: u64 = slots
        .iter()
        .filter(|(t, _)| **t != pubkey)
        .map(|(_, v)| v)
        .sum();
    Ok(global_counter_client::GlobalCounterClient {
        client,
        contract_key: key,
        slots,
        tag,
        pubkey,
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
        update_count_incrementally(&mut client, key, 0, 42)
            .await
            .unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 42);
        update_count_incrementally(&mut client, key, 0, 99)
            .await
            .unwrap();
        assert_eq!(get_count(&mut client, key).await.unwrap(), 99);
    }
}
