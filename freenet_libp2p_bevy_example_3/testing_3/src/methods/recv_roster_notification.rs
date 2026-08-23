use std::time::Duration;

use freenet_libp2p_bevy_example_3_lib::roster;
use freenet_stdlib::client_api::{ContractResponse, HostResponse};

pub async fn recv_roster_notification(
    client: &mut freenet_libp2p_bevy_example_3_lib::freenet::FreenetClient,
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
