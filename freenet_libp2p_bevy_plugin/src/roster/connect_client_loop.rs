use std::time::{Duration, SystemTime, UNIX_EPOCH};

use freenet_stdlib::client_api::{
    ClientRequest, ContractRequest, ContractResponse, HostResponse, NodeDiagnosticsConfig,
    NodeQuery, QueryResponse,
};
use freenet_stdlib::prelude::{ContractKey, State, UpdateData};

use super::roster_digest;
use crate::freenet;
use crate::net_id;
use crate::roster;

// needed helper:
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// needed helper:
async fn log_node_diagnostics(host: &str, port: u16) {
    let Ok(mut probe) = freenet::FreenetClient::connect(host, port).await else {
        return;
    };
    let request = ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics {
        config: NodeDiagnosticsConfig::basic_status(),
    });
    if probe.send(request).await.is_err() {
        return;
    }
    if let Some(Ok(HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(diag)))) =
        probe.recv_timeout(Duration::from_secs(5)).await
    {
        let active_connections = diag
            .network_info
            .as_ref()
            .map(|info| info.active_connections)
            .unwrap_or(0);
        let node_info_populated = diag.node_info.is_some();
        tracing::warn!(
            target: "roster",
            active_connections,
            node_info_populated,
            "node diagnostics at roster setup failure"
        );
    }
}

fn absorb(
    known: roster::RosterState,
    incoming: roster::RosterState,
    own_id: net_id::NetworkId,
) -> roster::RosterState {
    let own = known.get(&own_id).cloned();
    let merged = roster::merge_roster(known, incoming);
    let mut pruned = roster::prune_stale(merged, now_unix_secs(), roster::ROSTER_ENTRY_TTL_SECS);
    if let Some(own) = own {
        pruned.entry(own_id).or_insert(own);
    }
    pruned
}

async fn run_roster_loop(
    client: &mut freenet::FreenetClient,
    contract_key: ContractKey,
    own_id: net_id::NetworkId,
    own_entry: roster::PeerEntry,
    initial_entries: roster::RosterState,
    event_tx: tokio::sync::mpsc::UnboundedSender<roster::Event>,
) {
    let mut known: roster::RosterState = initial_entries;
    let mut heartbeat_deadline =
        tokio::time::Instant::now() + Duration::from_secs(roster::ROSTER_FAST_HEARTBEAT_SECS);
    let mut refresh_deadline = tokio::time::Instant::now() + Duration::from_millis(500);
    let mut resubscribe_deadline =
        tokio::time::Instant::now() + Duration::from_secs(roster::ROSTER_RESUBSCRIBE_SECS);
    let started = tokio::time::Instant::now();
    let instance_id = *contract_key.id();
    tracing::debug!(
        target: "roster",
        contract = %contract_key,
        own_id = format!("{:08x}", *own_id as u32),
        digest = %roster_digest::roster_digest(&known),
        "roster loop started"
    );
    loop {
        let fast_window =
            started.elapsed() < Duration::from_secs(roster::ROSTER_FAST_REFRESH_WINDOW_SECS);
        let heartbeat_interval = if fast_window {
            Duration::from_secs(roster::ROSTER_FAST_HEARTBEAT_SECS)
        } else {
            Duration::from_secs(roster::ROSTER_HEARTBEAT_SECS)
        };
        let refresh_interval = if fast_window {
            Duration::from_secs(roster::ROSTER_FAST_REFRESH_SECS)
        } else {
            Duration::from_secs(roster::ROSTER_REFRESH_SECS)
        };
        let heartbeat_wait = tokio::time::sleep_until(heartbeat_deadline);
        let refresh_wait = tokio::time::sleep_until(refresh_deadline);
        tokio::pin!(heartbeat_wait);
        tokio::pin!(refresh_wait);
        tokio::select! {
            recv = client.recv() => match recv {
                Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                    update,
                    ..
                })) => {
                    if let Some(entries) = roster::decode_roster_update(&update) {
                        let incoming = roster_digest::roster_digest(&entries);
                        known = absorb(known, entries, own_id);
                        tracing::info!(
                            target: "roster",
                            incoming = %incoming,
                            digest = %roster_digest::roster_digest(&known),
                            "received roster UpdateNotification"
                        );
                        event_tx.send(roster::Event::Roster {
                            entries: known.clone(),
                        }).ok();
                    }
                }
                Ok(HostResponse::ContractResponse(ContractResponse::GetResponse {
                    state,
                    ..
                })) => {
                    let entries: roster::RosterState =
                        match bincode::deserialize(state.as_ref()) {
                            Ok(entries) => entries,
                            Err(_) => continue,
                        };
                    let incoming = roster_digest::roster_digest(&entries);
                    known = absorb(known, entries, own_id);
                    tracing::info!(
                        target: "roster",
                        incoming = %incoming,
                        digest = %roster_digest::roster_digest(&known),
                        "received roster GetResponse (refresh)"
                    );
                    event_tx.send(roster::Event::Roster {
                        entries: known.clone(),
                    }).ok();
                }
                Ok(_) => continue,
                Err(_) => break,
            },
            _ = &mut heartbeat_wait => {
                let mut own = own_entry.clone();
                own.updated_at = now_unix_secs();
                known.insert(own_id, own);
                known = roster::prune_stale(
                    known,
                    now_unix_secs(),
                    roster::ROSTER_ENTRY_TTL_SECS,
                );
                let refreshed = known.clone();
                let Ok(bytes) = bincode::serialize(&refreshed) else {
                    heartbeat_deadline = tokio::time::Instant::now() + heartbeat_interval;
                    continue;
                };
                tracing::debug!(
                    target: "roster",
                    digest = %roster_digest::roster_digest(&refreshed),
                    bytes = bytes.len(),
                    "sending roster heartbeat Update"
                );
                let update_req = ContractRequest::Update {
                    key: contract_key,
                    data: UpdateData::State(State::from(bytes)),
                };
                if let Err(e) = client.send(ClientRequest::ContractOp(update_req)).await {
                    tracing::warn!(target: "roster", error = %e, "heartbeat update failed");
                }
                event_tx.send(roster::Event::Roster { entries: refreshed }).ok();
                heartbeat_deadline = tokio::time::Instant::now() + heartbeat_interval;
            }
            _ = &mut refresh_wait => {
                let resubscribe = tokio::time::Instant::now() >= resubscribe_deadline;
                let get_req = ContractRequest::Get {
                    key: instance_id,
                    return_contract_code: false,
                    subscribe: resubscribe,
                    blocking_subscribe: false,
                };
                tracing::trace!(
                    target: "roster",
                    resubscribe,
                    digest = %roster_digest::roster_digest(&known),
                    "sending roster refresh Get"
                );
                if let Err(e) = client.send(ClientRequest::ContractOp(get_req)).await {
                    tracing::warn!(target: "roster", error = %e, "refresh get failed");
                }
                if resubscribe {
                    resubscribe_deadline = tokio::time::Instant::now()
                        + Duration::from_secs(roster::ROSTER_RESUBSCRIBE_SECS);
                }
                refresh_deadline = tokio::time::Instant::now() + refresh_interval;
            }
        }
    }
}

pub async fn connect_client_loop(
    args: roster::ConnectClientArgs<'_>,
    event_tx: tokio::sync::mpsc::UnboundedSender<roster::Event>,
) {
    let roster::ConnectClientArgs {
        host,
        port,
        contract_wasm,
        params,
        own_id,
        own_entry,
        not_found_grace,
    } = args;
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        event_tx.send(roster::Event::Connecting { attempt }).ok();

        let mut entry = own_entry.clone();
        entry.updated_at = now_unix_secs();

        match roster::setup_contract(
            host,
            port,
            contract_wasm,
            params,
            own_id,
            entry.clone(),
            not_found_grace,
        )
        .await
        {
            Ok((mut client, contract_key, entries)) => {
                event_tx
                    .send(roster::Event::Roster {
                        entries: entries.clone(),
                    })
                    .ok();
                run_roster_loop(
                    &mut client,
                    contract_key,
                    own_id,
                    entry,
                    entries,
                    event_tx.clone(),
                )
                .await;
            }
            Err(e) => {
                log_node_diagnostics(host, port).await;
                event_tx
                    .send(roster::Event::ConnectionError(format!("setup failed: {e}")))
                    .ok();
            }
        }

        let backoff = (roster::SETUP_RETRY_BACKOFF_SECS * attempt as u64)
            .min(roster::SETUP_RETRY_MAX_BACKOFF_SECS);
        tracing::info!(target: "roster", attempt, backoff, "retrying roster setup");
        tokio::time::sleep(Duration::from_secs(backoff)).await;
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
