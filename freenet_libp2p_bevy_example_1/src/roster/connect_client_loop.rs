use std::time::{Duration, SystemTime, UNIX_EPOCH};

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::{State, UpdateData};

use crate::boxes;
use crate::roster;

// needed helper:
fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Connects to an already-running embedded node's WebSocket API, deploys/joins the roster
/// contract under the given `params` (unique params = a private contract instance for a
/// test), and forwards roster changes to the game app until the client drops.
pub async fn connect_client_loop(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
    event_tx: tokio::sync::mpsc::UnboundedSender<roster::Event>,
) {
    let (mut client, contract_key, entries) =
        match roster::setup_contract(host, port, contract_wasm, params, own_id, own_entry.clone())
            .await
        {
            Ok(r) => r,
            Err(e) => {
                event_tx
                    .send(roster::Event::ConnectionError(format!("setup failed: {e}")))
                    .ok();
                return;
            }
        };

    event_tx.send(roster::Event::Roster { entries }).ok();

    let mut heartbeat = tokio::time::interval(Duration::from_secs(roster::ROSTER_HEARTBEAT_SECS));
    loop {
        tokio::select! {
            recv = client.recv() => match recv {
                Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
                    update,
                    ..
                })) => {
                    if let Some(entries) = roster::decode_roster_update(&update) {
                        tracing::info!(
                            target: "roster",
                            entries = entries.len(),
                            "received roster UpdateNotification"
                        );
                        event_tx.send(roster::Event::Roster { entries }).ok();
                    }
                }
                Ok(_) => continue,
                Err(_) => break,
            },
            _ = heartbeat.tick() => {
                let mut refreshed = own_entry.clone();
                refreshed.updated_at = now_unix_secs();
                let mut own_roster = roster::RosterState::default();
                own_roster.insert(own_id, refreshed);
                let Ok(bytes) = bincode::serialize(&own_roster) else {
                    continue;
                };
                let update_req = ContractRequest::Update {
                    key: contract_key,
                    data: UpdateData::State(State::from(bytes)),
                };
                if let Err(e) = client.send(ClientRequest::ContractOp(update_req)).await {
                    tracing::warn!(target: "roster", error = %e, "heartbeat update failed");
                }
            }
        }
    }
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
