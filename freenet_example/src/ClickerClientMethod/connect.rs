use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::ClickerError;
use crate::ClickerError as Ce;
use crate::FreenetClient;
use crate::Role;
use crate::recv_response;

pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    role: Role,
) -> Result<crate::ClickerClient, ClickerError> {
    let mut client = FreenetClient::connect(host, port).await?;

    let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let params = Parameters::from(Vec::new());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();

    let (key, count) = match role {
        Role::Publish => {
            let result = crate::recv_after_get(&mut client, instance_id).await;
            match result {
                Ok(r) => r,
                Err(_) => {
                    let put_req = ContractRequest::Put {
                        contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped)),
                        state: WrappedState::new(bincode::serialize(&0u64)?),
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
                    crate::recv_after_get(&mut client, instance_id).await?
                }
            }
        }
        Role::Subscribe => loop {
            match crate::recv_after_get(&mut client, instance_id).await {
                Ok(r) => break r,
                Err(_) => {
                    info!(
                        target: "freenet_example",
                        %instance_id,
                        "contract not found, retrying in 1s"
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        },
    };

    Ok(crate::ClickerClient {
        client,
        contract_key: key,
        count,
    })
}

#[cfg(test)]
mod tests {
    use crate::testing::*;
    use crate::ClickerClient;
    use crate::Role;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let mut client = connect(node.port()).await;
        let key = deploy(&mut client, &wasm).await;
        assert_eq!(get_count(&mut client, key).await, 0);
        update_count(&mut client, key, 42).await;
        assert_eq!(get_count(&mut client, key).await, 42);
        update_count(&mut client, key, 99).await;
        assert_eq!(get_count(&mut client, key).await, 99);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_persistence() {
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let key;
        {
            let mut client = connect(node.port()).await;
            key = deploy(&mut client, &wasm).await;
            update_count(&mut client, key, 5).await;
            assert_eq!(get_count(&mut client, key).await, 5);
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        {
            let mut client = connect(node.port()).await;
            assert_eq!(get_count(&mut client, key).await, 5);
            update_count(&mut client, key, 8).await;
            assert_eq!(get_count(&mut client, key).await, 8);
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_publish_subscribe() {
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let mut pub_ = connect(node.port()).await;
        let key = deploy(&mut pub_, &wasm).await;
        update_count(&mut pub_, key, 5).await;
        let mut sub = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Subscribe)
            .await
            .unwrap();
        assert_eq!(sub.state().await.unwrap(), 5);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_standalone_demo_works() {
        let node = TestNode::start().await;
        let wasm = load_wasm();
        let mut clicker = ClickerClient::connect("127.0.0.1", node.port(), &wasm, Role::Publish)
            .await
            .unwrap();
        assert!(clicker.count() == 0);
        for expected in 1..=3 {
            assert_eq!(clicker.tick().await.unwrap(), expected);
        }
        assert_eq!(clicker.state().await.unwrap(), 3);
    }
}
