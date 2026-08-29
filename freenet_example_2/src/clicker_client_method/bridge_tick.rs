use crate::clicker_client;
use crate::clicker_client_method;
use crate::clicker_error;
use std::collections::BTreeMap;
use std::time::Duration;

use freenet_stdlib::client_api::{ClientRequest, ContractRequest, ContractResponse, HostResponse};
use freenet_stdlib::prelude::*;
use tracing::info;

const SPLIT_AFTER_SECS: u64 = 30;
const BRIDGE_INTERVAL_SECS: u64 = 30;
const SUBSCRIBE_TIMEOUT: Duration = Duration::from_secs(30);
const REPUT_TIMEOUT: Duration = Duration::from_secs(60);

pub async fn bridge_tick(
    client: &mut clicker_client::ClickerClient,
) -> Result<(), clicker_error::ClickerError> {
    if !bridge_due(
        client.foreign_seen,
        client.last_bridge,
        std::time::Instant::now(),
    ) {
        return Ok(());
    }
    client.last_bridge = Some(std::time::Instant::now());
    info!(target: "freenet_example", tag = client.tag, "bridge: split suspected");

    let merged_now = attempt_subscribe(client).await?;
    clicker_client_method::note_foreign_slots(client);
    if merged_now {
        info!(target: "freenet_example", tag = client.tag, "bridge: merged via subscribe");
        return Ok(());
    }

    let merged_now = attempt_reput(client).await?;
    clicker_client_method::note_foreign_slots(client);
    if merged_now {
        info!(target: "freenet_example", tag = client.tag, "bridge: merged via re-put");
    }
    Ok(())
}

// needed helper:
fn bridge_due(
    foreign_seen: Option<std::time::Instant>,
    last_bridge: Option<std::time::Instant>,
    now: std::time::Instant,
) -> bool {
    let silent = foreign_seen.is_none_or(|t| now.duration_since(t).as_secs() >= SPLIT_AFTER_SECS);
    let due = last_bridge.is_none_or(|t| now.duration_since(t).as_secs() >= BRIDGE_INTERVAL_SECS);
    silent && due
}

// needed helper:
async fn attempt_subscribe(
    client: &mut clicker_client::ClickerClient,
) -> Result<bool, clicker_error::ClickerError> {
    info!(target: "freenet_example", tag = client.tag, "bridge: subscribe attempt");
    let instance_id = *client.contract_key.id();
    let summary = StateSummary::from(bincode::serialize(&client.slots)?);
    let sub_req = ContractRequest::Subscribe {
        key: instance_id,
        summary: Some(summary),
    };
    client
        .client
        .send(ClientRequest::ContractOp(sub_req))
        .await?;
    let response = tokio::time::timeout(SUBSCRIBE_TIMEOUT, client.client.recv_response()).await;
    match response {
        Err(_) => {
            info!(target: "freenet_example", tag = client.tag, "bridge: subscribe timed out");
            Ok(client.foreign_seen.is_some())
        }
        Ok(r) => {
            info!(target: "freenet_example", tag = client.tag, "bridge: subscribe response={}", response_kind(&r?));
            Ok(client.foreign_seen.is_some())
        }
    }
}

// needed helper:
fn response_kind(response: &HostResponse) -> &'static str {
    match response {
        HostResponse::ContractResponse(ContractResponse::SubscribeResponse { .. }) => "subscribed",
        HostResponse::ContractResponse(ContractResponse::GetResponse { .. }) => "state",
        HostResponse::ContractResponse(ContractResponse::UpdateNotification { .. }) => {
            "notification"
        }
        HostResponse::ContractResponse(ContractResponse::UpdateResponse { .. }) => "update",
        HostResponse::ContractResponse(ContractResponse::NotFound { .. }) => "not-found",
        _ => "other",
    }
}

// needed helper:
async fn attempt_reput(
    client: &mut clicker_client::ClickerClient,
) -> Result<bool, clicker_error::ClickerError> {
    info!(target: "freenet_example", tag = client.tag, "bridge: re-put attempt");
    let state = WrappedState::new(bincode::serialize(&client.slots)?);
    let put_req = ContractRequest::Put {
        contract: client.contract.clone(),
        state,
        related_contracts: RelatedContracts::default(),
        subscribe: true,
        blocking_subscribe: false,
    };
    client
        .client
        .send(ClientRequest::ContractOp(put_req))
        .await?;
    let response = tokio::time::timeout(REPUT_TIMEOUT, client.client.recv_response()).await;
    let response = match response {
        Ok(r) => r?,
        Err(_) => {
            info!(target: "freenet_example", tag = client.tag, "bridge: re-put timed out");
            return Ok(client.foreign_seen.is_some());
        }
    };
    match response {
        HostResponse::ContractResponse(
            ContractResponse::PutResponse { .. }
            | ContractResponse::SubscribeResponse { .. }
            | ContractResponse::UpdateResponse { .. },
        ) => {
            info!(target: "freenet_example", tag = client.tag, "bridge: re-put response=put");
        }
        HostResponse::ContractResponse(ContractResponse::UpdateNotification { update, .. }) => {
            let bytes = match update {
                UpdateData::State(s) => Some(s.as_ref().to_vec()),
                UpdateData::Delta(d) => Some(d.as_ref().to_vec()),
                _ => None,
            };
            if let Some(bytes) = bytes
                && let Ok(incoming) = bincode::deserialize::<BTreeMap<u64, u64>>(&bytes)
            {
                clicker_client_method::merge_slots(&mut client.slots, incoming);
            }
        }
        other => {
            return Err(clicker_error::ClickerError::UnexpectedResponse(format!(
                "{other:?}"
            )));
        }
    }
    Ok(client.foreign_seen.is_some())
}

#[cfg(test)]
mod tests {
    use super::bridge_due;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    fn test_usage() {
        let now = Instant::now();
        assert!(bridge_due(None, None, now));
        assert!(!bridge_due(Some(now), None, now));

        let stale = now - Duration::from_secs(120);
        assert!(bridge_due(Some(stale), None, now));
        assert!(bridge_due(Some(stale), Some(stale), now));

        let fresh = now - Duration::from_secs(1);
        assert!(!bridge_due(Some(fresh), Some(fresh), now));
    }
}
