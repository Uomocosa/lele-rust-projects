use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

/// # Errors
/// Returns `Error` if the connection or directory get/put fails.
pub async fn connect_directory(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
) -> Result<discovery::Directory, discovery::Error> {
    let mut client = discovery::Client::connect(host, port).await?;
    let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let params = Parameters::from(params.to_vec());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    let initial = discovery::DirectoryState::new();
    let (key, slots) = match discovery::recv_after_get_directory(&mut client, instance_id).await {
        Ok(found) => found,
        Err(discovery::Error::ContractNotFound) => {
            let put_req = ContractRequest::Put {
                contract: container.clone(),
                state: WrappedState::new(bincode::serialize(&initial)?),
                related_contracts: RelatedContracts::default(),
                subscribe: true,
                blocking_subscribe: false,
            };
            client.send(&ClientRequest::ContractOp(put_req))?;
            match client.recv_response().await? {
                HostResponse::ContractResponse(
                    ContractResponse::PutResponse { key }
                    | ContractResponse::SubscribeResponse { key, .. }
                    | ContractResponse::UpdateResponse { key, .. },
                ) => {
                    info!(target: "clicker", key = %key, "directory contract deployed");
                }
                other => {
                    return Err(discovery::Error::UnexpectedResponse(format!("{other:?}")));
                }
            }
            discovery::recv_after_get_directory(&mut client, instance_id).await?
        }
        Err(e) => return Err(e),
    };
    Ok(discovery::Directory {
        client,
        contract_key: key,
        contract: container,
        slots,
    })
}
// no test_usage necessary
