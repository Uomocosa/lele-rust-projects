use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::boxes;
use crate::freenet;
use crate::roster;

// needed helper:
async fn recv_timeout(client: &mut freenet::FreenetClient) -> Result<HostResponse, String> {
    match client.recv_response_timeout(Duration::from_secs(60)).await {
        Some(Ok(r)) => Ok(r),
        Some(Err(e)) => Err(format!("{e}")),
        None => Err("timeout after 60s".into()),
    }
}

/// Connects to the embedded node, deploys the roster contract if missing, merges in this
/// player's own entry, and returns the connected client plus the merged roster.
pub async fn setup_contract(
    host: &str,
    port: u16,
    wasm: &[u8],
    params: &[u8],
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
) -> Result<(freenet::FreenetClient, ContractKey, roster::RosterState), String> {
    let mut client = loop {
        match freenet::FreenetClient::connect(host, port).await {
            Ok(c) => break c,
            Err(_) => {
                tracing::info!(target: "roster", "connect failed, retrying in 1s");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    };

    let code = Arc::new(ContractCode::from(wasm.to_vec()));
    let contract_params = Parameters::from(params.to_vec());
    let wrapped = WrappedContract::new(code, contract_params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();

    let mut own_roster = roster::RosterState::default();
    own_roster.insert(own_id, own_entry);

    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client
        .send(ClientRequest::ContractOp(get_req))
        .await
        .map_err(|e| format!("send get: {e}"))?;

    let roster = loop {
        match recv_timeout(&mut client).await? {
            HostResponse::ContractResponse(ContractResponse::GetResponse { state, .. }) => {
                let existing: roster::RosterState =
                    bincode::deserialize(state.as_ref()).map_err(|e| format!("deser: {e}"))?;
                if existing.contains_key(&own_id) {
                    break existing;
                }
                let merged = roster::merge_roster(existing, own_roster.clone());
                let update_req = ContractRequest::Update {
                    key: contract_key,
                    data: UpdateData::State(State::from(
                        bincode::serialize(&merged).map_err(|e| format!("ser: {e}"))?,
                    )),
                };
                client
                    .send(ClientRequest::ContractOp(update_req))
                    .await
                    .map_err(|e| format!("send update: {e}"))?;
                recv_timeout(&mut client).await?;
                break merged;
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                let put_req = ContractRequest::Put {
                    contract: ContractContainer::from(ContractWasmAPIVersion::V1(wrapped.clone())),
                    state: WrappedState::new(
                        bincode::serialize(&own_roster).map_err(|e| format!("ser: {e}"))?,
                    ),
                    related_contracts: RelatedContracts::default(),
                    subscribe: true,
                    blocking_subscribe: false,
                };
                client
                    .send(ClientRequest::ContractOp(put_req))
                    .await
                    .map_err(|e| format!("send put: {e}"))?;
                recv_timeout(&mut client).await?;
                break own_roster.clone();
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => continue,
            HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => continue,
            other => return Err(format!("unexpected: {other:?}")),
        }
    };

    Ok((client, contract_key, roster))
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
