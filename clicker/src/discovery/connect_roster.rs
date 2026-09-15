use std::sync::Arc;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

/// # Errors
/// Returns `Error` if the connection or contract get/put fails.
pub async fn connect_roster(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    own: discovery::PlayerId,
    peer_id: &str,
    addrs: &[String],
) -> Result<discovery::Roster, discovery::Error> {
    let mut client = discovery::Client::connect(host, port).await?;
    let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let params = Parameters::from(params.to_vec());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    let own_entry = discovery::PeerEntry {
        peer_id: peer_id.to_string(),
        addrs: addrs.to_vec(),
        updated_at: now_secs(),
    };
    let mut initial = discovery::RosterState::new();
    initial.insert(own, own_entry);
    let (key, slots) = match discovery::recv_after_get(&mut client, instance_id).await {
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
                    info!(target: "clicker", key = %key, "roster contract deployed");
                }
                other => {
                    return Err(discovery::Error::UnexpectedResponse(format!("{other:?}")));
                }
            }
            discovery::recv_after_get(&mut client, instance_id).await?
        }
        Err(e) => return Err(e),
    };
    let roster = discovery::Roster {
        client,
        contract_key: key,
        contract: container,
        slots,
        own,
        peer_id: peer_id.to_string(),
        addrs: addrs.to_vec(),
        foreign_seen: None,
        foreign_sum: 0,
        last_bridge: None,
    };
    Ok(refresh_foreign(roster))
}

// needed helper: seconds since the unix epoch for entry timestamps
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

// needed helper: recomputes the foreign-entry baseline after connect
fn refresh_foreign(mut roster: discovery::Roster) -> discovery::Roster {
    let own = roster.own;
    let sum: u64 = roster
        .slots
        .iter()
        .filter(|(id, _)| **id != own)
        .map(|(_, e)| e.updated_at)
        .sum();
    roster.foreign_sum = sum;
    if sum > 0 {
        roster.foreign_seen = Some(std::time::Instant::now());
    }
    roster
}
// no test_usage necessary
