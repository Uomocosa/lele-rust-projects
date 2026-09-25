use std::time::{Duration, Instant};

use tracing::warn;

use super::super::params::contract_params::ContractParams;
use super::super::params::remote_peer_id::RemotePeerId;
use super::super::params::room_name::RoomName;
use super::super::params::room_params::room_params;
use super::super::params::unique_game_id::UniqueGameId;
use super::catalog_client::CatalogClient;
use super::dial_directory_publishers::dial_directory_publishers;
use super::fire_due_staggers_raw::fire_due_staggers_raw;
use super::maps::{AttemptedMap, ConnectedMap, StaggerMap};
use super::run_config::RunConfig;
use std::collections::HashMap;

#[must_use]
pub async fn resolve_room(
    config: &mut RunConfig,
    directory: &mut CatalogClient,
    peer_id: &RemotePeerId,
    addrs: &[String],
) -> Option<(RoomName, ContractParams)> {
    let deadline = Instant::now().checked_add(Duration::from_secs(config.timing.timeout_secs))?;
    let mut attempted: AttemptedMap = HashMap::new();
    let connected: ConnectedMap = HashMap::new();
    let mut staggers: StaggerMap = HashMap::new();
    loop {
        if let Ok(room) = config.room_requests.try_recv() {
            return Some(resolve_requested(
                directory, &config.id, room, peer_id, addrs,
            ));
        }
        match directory.poll().await {
            Ok(slots) => {
                config.directory_tx.send(slots.clone()).ok();
                dial_directory_publishers(
                    &config.net_tx,
                    &mut attempted,
                    &connected,
                    &mut staggers,
                    peer_id.as_str(),
                    config.transport,
                    &slots,
                );
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: catalog poll failed");
            }
        }
        if Instant::now() >= deadline {
            return None;
        }
        fire_due_staggers_raw(&config.net_tx, &connected, &mut staggers);
        let request = tokio::select! {
            biased;
            request = config.room_requests.recv() => request,
            () = tokio::time::sleep(Duration::from_secs(config.timing.tick_secs)) => None,
        };
        if let Some(room) = request {
            return Some(resolve_requested(
                directory, &config.id, room, peer_id, addrs,
            ));
        }
    }
}

// needed helper: publishes a user-requested room and resolves its join params
fn resolve_requested(
    directory: &CatalogClient,
    id: &UniqueGameId,
    room: RoomName,
    peer_id: &RemotePeerId,
    addrs: &[String],
) -> (RoomName, ContractParams) {
    let params = room_params(id, &room);
    if directory
        .publish_room(&room, &params, peer_id, addrs)
        .is_err()
    {
        warn!(target: "room_lobby", "discovery: room publish failed");
    }
    (room, params)
}

#[cfg(test)]
mod tests {
    use super::room_params;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let id = discovery::params::UniqueGameId::new(
            &discovery::params::GameName("test".to_string()),
            "token",
        );
        let room = discovery::params::RoomName("room-a".to_string());
        let params = room_params(&id, &room);
        let decoded: (String, String) = bincode::deserialize(&params).unwrap_or_default();
        assert_eq!(decoded.0, "test/token");
        assert_eq!(decoded.1, "room-a");
    }
}
