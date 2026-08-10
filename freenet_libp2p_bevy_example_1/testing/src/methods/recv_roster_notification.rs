use std::time::Duration;

use bevy_freenet::roster;
use freenet_stdlib::client_api::{ContractResponse, HostResponse};

pub async fn recv_roster_notification(
    client: &mut bevy_freenet::freenet::FreenetClient,
    timeout: Duration,
) -> Option<roster::RosterState> {
    match tokio::time::timeout(timeout, client.recv()).await {
        Ok(Ok(HostResponse::ContractResponse(ContractResponse::UpdateNotification {
            update,
            ..
        }))) => roster::decode_roster_update(&update),
        _ => None,
    }
}
