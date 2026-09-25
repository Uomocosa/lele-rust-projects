use std::sync::Arc;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

use crate::discovery;

pub async fn connect(
    host: &str,
    port: u16,
    contract_wasm: &[u8],
    params: &[u8],
    own: discovery::params::PlayerId,
    peer_id: &discovery::params::RemotePeerId,
    addrs: &[String],
) -> Result<discovery::link::RosterClient, discovery::Error> {
    let mut client = discovery::link::Client::connect(host, port).await?;
    let contract_code = Arc::new(ContractCode::from(contract_wasm.to_vec()));
    let params = Parameters::from(params.to_vec());
    let wrapped = WrappedContract::new(contract_code, params);
    let contract_key = wrapped.key;
    let instance_id = *contract_key.id();
    let container = ContractContainer::from(ContractWasmAPIVersion::V1(wrapped));
    let own_entry = discovery::membership::PeerEntry {
        peer_id: peer_id.clone(),
        addrs: addrs.to_vec(),
        updated_at: discovery::params::EpochSecs(now_secs()),
    };
    let mut initial = discovery::membership::RosterState::new();
    initial.insert(own, own_entry);
    let (key, slots) = match recv_after_get(&mut client, instance_id).await {
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
                    info!(target: "room_lobby", key = %key, "roster contract deployed");
                }
                other => {
                    return Err(discovery::Error::UnexpectedResponse(format!("{other:?}")));
                }
            }
            recv_after_get(&mut client, instance_id).await?
        }
        Err(e) => return Err(e),
    };
    let roster = discovery::link::RosterClient {
        client,
        contract_key: key,
        contract: container,
        slots,
        own,
        peer_id: peer_id.clone(),
        addrs: addrs.to_vec(),
        foreign_seen: None,
        foreign_sum: 0,
        last_bridge: None,
    };
    Ok(refresh_foreign(roster))
}

// needed helper: recomputes the foreign-entry baseline after connect
fn refresh_foreign(mut roster: discovery::link::RosterClient) -> discovery::link::RosterClient {
    let own = roster.own;
    let sum: u64 = roster
        .slots
        .iter()
        .filter(|(id, _)| **id != own)
        .map(|(_, e)| *e.updated_at)
        .sum();
    roster.foreign_sum = sum;
    if sum > 0 {
        roster.foreign_seen = Some(std::time::Instant::now());
    }
    roster
}

// needed helper: issues a blocking subscribe-get and reads the first state
async fn recv_after_get(
    client: &mut discovery::link::Client,
    instance_id: ContractInstanceId,
) -> Result<(ContractKey, discovery::membership::RosterState), discovery::Error> {
    let get_req = ContractRequest::Get {
        key: instance_id,
        return_contract_code: false,
        subscribe: true,
        blocking_subscribe: true,
    };
    client.send(&ClientRequest::ContractOp(get_req))?;
    let wait = Duration::from_secs(discovery::CONNECT_TIMEOUT_SECS);
    loop {
        let response = tokio::time::timeout(wait, client.recv_response())
            .await
            .map_err(|_| discovery::Error::ResponseTimeout)?;
        match response? {
            HostResponse::ContractResponse(ContractResponse::GetResponse {
                key, state, ..
            }) => {
                let slots = bincode::deserialize(state.as_ref()).unwrap_or_default();
                return Ok((key, slots));
            }
            HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => {
                return Err(discovery::Error::ContractNotFound);
            }
            HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => {}
            other => return Err(discovery::Error::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}

// needed helper: seconds since the unix epoch for entry timestamps
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::connect;
    use crate::discovery;

    #[tokio::test]
    async fn test_usage() {
        let result = connect(
            "127.0.0.1",
            1,
            &[],
            &[],
            discovery::params::PlayerId(0),
            &discovery::params::RemotePeerId("peer".to_string()),
            &[],
        )
        .await;
        assert!(result.is_err());
    }
}
