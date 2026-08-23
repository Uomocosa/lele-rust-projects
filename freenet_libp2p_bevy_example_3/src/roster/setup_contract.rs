use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;

use crate::boxes;
use crate::freenet;
use crate::roster;

// needed helper:
async fn recv_timeout(client: &mut freenet::FreenetClient) -> Result<Option<HostResponse>, String> {
    match client.recv_response_timeout(Duration::from_secs(30)).await {
        Some(Ok(r)) => Ok(Some(r)),
        Some(Err(e)) => Err(format!("{e}")),
        None => Ok(None),
    }
}

// needed helper:
async fn recheck_get(
    client: &mut freenet::FreenetClient,
    instance_id: freenet_stdlib::prelude::ContractInstanceId,
) -> Result<(), String> {
    let reget = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: false,
    };
    client
        .send(ClientRequest::ContractOp(reget))
        .await
        .map_err(|e| format!("send re-get: {e}"))?;
    tokio::time::sleep(Duration::from_secs(
        roster::SETUP_CONTRACT_CHECK_INTERVAL_SECS,
    ))
    .await;
    Ok(())
}

/// Connects to the embedded node, deploys the roster contract if missing, merges in this
/// player's own entry, and returns the connected client plus the merged roster.
///
/// `not_found_grace` guards the deploy: while a `Get` neither finds the contract nor returns
/// `NotFound` — the contract has not been seeded yet, or the mainnet ring is too degraded to
/// answer — setup re-`Get`s every `SETUP_CONTRACT_CHECK_INTERVAL_SECS` until the grace window
/// expires, then `Put`s. A `Duration::ZERO` grace Puts immediately on the first miss
/// (hermetic/isolated-host behavior). The grace exists because two users racing the FIRST
/// `Put` of a brand-new key can seed two disjoint replicas that only reconcile via freenet's
/// 5-minute InterestSync anti-entropy (see OBJECTIVE.md); a staggered re-check makes the
/// second joiner find the first joiner's seed instead of racing it.
pub async fn setup_contract(
    host: &str,
    port: u16,
    wasm: &[u8],
    params: &[u8],
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
    not_found_grace: Duration,
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

    let grace_deadline = tokio::time::Instant::now() + not_found_grace;
    tracing::debug!(
        target: "roster",
        contract = %contract_key,
        own_id = %hex::encode(own_id),
        grace_secs = not_found_grace.as_secs(),
        "roster setup starting"
    );

    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: false,
    };
    client
        .send(ClientRequest::ContractOp(get_req))
        .await
        .map_err(|e| format!("send get: {e}"))?;

    let roster = loop {
        match recv_timeout(&mut client).await? {
            Some(HostResponse::ContractResponse(ContractResponse::GetResponse {
                state, ..
            })) => {
                let existing: roster::RosterState =
                    bincode::deserialize(state.as_ref()).map_err(|e| format!("deser: {e}"))?;
                tracing::info!(
                    target: "roster",
                    digest = %roster::roster_digest(&existing),
                    already_present = existing.contains_key(&own_id),
                    "roster GetResponse"
                );
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
                tracing::info!(
                    target: "roster",
                    digest = %roster::roster_digest(&merged),
                    "merging own entry, sending roster Update"
                );
                client
                    .send(ClientRequest::ContractOp(update_req))
                    .await
                    .map_err(|e| format!("send update: {e}"))?;
                match recv_timeout(&mut client).await? {
                    Some(_) => {}
                    None => return Err("update confirmation timed out".into()),
                }
                break merged;
            }
            Some(HostResponse::ContractResponse(ContractResponse::NotFound { .. })) => {
                if tokio::time::Instant::now() < grace_deadline {
                    tracing::info!(
                        target: "roster",
                        "contract not found yet, re-checking within grace window"
                    );
                    recheck_get(&mut client, instance_id).await?;
                    continue;
                }
                tracing::info!(
                    target: "roster",
                    digest = %roster::roster_digest(&own_roster),
                    "grace window expired, sending roster Put"
                );
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
                match recv_timeout(&mut client).await? {
                    Some(_) => {}
                    None => return Err("put confirmation timed out".into()),
                }
                break own_roster.clone();
            }
            Some(HostResponse::ContractResponse(ContractResponse::SubscribeResponse {
                ..
            })) => continue,
            Some(HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. })) => {
                continue;
            }
            Some(other) => return Err(format!("unexpected: {other:?}")),
            None => {
                if tokio::time::Instant::now() < grace_deadline {
                    tracing::info!(
                        target: "roster",
                        "get op timed out, re-checking within grace window"
                    );
                    recheck_get(&mut client, instance_id).await?;
                    continue;
                }
                return Err("timeout after 30s".into());
            }
        }
    };

    Ok((client, contract_key, roster))
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
