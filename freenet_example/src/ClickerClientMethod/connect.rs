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
    #[test]
    fn test_usage() {}
}
