use std::collections::HashMap;
use std::time::{Duration, Instant};

use tracing::warn;

use super::super::constants;
use super::super::dial::auto_join::auto_join;
use super::super::directory::Entry;
use super::super::directory::pick_room::pick_room;
use super::super::params::resolve_params::resolve_params;
use super::dial_hint_raw::dial_hint_raw;
use super::directory_client::DirectoryClient;
use super::directory_hints::directory_hints;
use super::fire_due_staggers_raw::fire_due_staggers_raw;
use super::maps::{ConnectedMap, StaggerMap};
use super::run_config::RunConfig;

#[must_use]
pub async fn resolve_room(
    config: &mut RunConfig,
    directory: &mut DirectoryClient,
    peer_id: &str,
    addrs: &[String],
) -> Option<(String, Vec<u8>)> {
    if let Some(room) = config.lobby.as_deref() {
        let params = resolve_params(&config.namespace, room, config.params_override.as_deref());
        if directory
            .publish_room(room, &params, peer_id, addrs)
            .is_err()
        {
            warn!(target: "room_lobby", "discovery: room publish failed");
        }
        return Some((room.to_string(), params));
    }
    let deadline = Instant::now().checked_add(Duration::from_secs(constants::DISCOVERY_SECS))?;
    let mut attempted: HashMap<String, Instant> = HashMap::new();
    let connected: ConnectedMap = HashMap::new();
    let mut staggers: StaggerMap = HashMap::new();
    loop {
        if let Ok(room) = config.room_requests.try_recv() {
            return Some(resolve_requested(
                directory,
                &config.namespace,
                config.params_override.as_deref(),
                room,
                peer_id,
                addrs,
            ));
        }
        match directory.poll().await {
            Ok(slots) => {
                config.directory_tx.send(slots.clone()).ok();
                for hint in directory_hints(&slots) {
                    dial_hint_raw(
                        &config.net_tx,
                        &mut attempted,
                        &connected,
                        &mut staggers,
                        peer_id,
                        config.transport,
                        &hint,
                    );
                }
                if auto_join(std::env::var("ROOM_LOBBY_NO_AUTOJOIN").ok().map(|_| true))
                    && let Some((room, entry)) = pick_room(&slots, config.since_secs)
                {
                    let params = room_params(
                        &config.namespace,
                        &room,
                        config.params_override.as_deref(),
                        entry,
                    );
                    return Some((room, params));
                }
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: room poll failed");
            }
        }
        if Instant::now() >= deadline {
            return None;
        }
        fire_due_staggers_raw(&config.net_tx, &connected, &mut staggers);
        let request = tokio::select! {
            biased;
            request = config.room_requests.recv() => request,
            () = tokio::time::sleep(Duration::from_secs(constants::DIRECTORY_TICK_SECS)) => None,
        };
        if let Some(room) = request {
            return Some(resolve_requested(
                directory,
                &config.namespace,
                config.params_override.as_deref(),
                room,
                peer_id,
                addrs,
            ));
        }
    }
}

// needed helper: publishes a user-requested room and resolves its join params
fn resolve_requested(
    directory: &DirectoryClient,
    namespace: &str,
    params_override: Option<&str>,
    room: String,
    peer_id: &str,
    addrs: &[String],
) -> (String, Vec<u8>) {
    let params = resolve_params(namespace, &room, params_override);
    if directory
        .publish_room(&room, &params, peer_id, addrs)
        .is_err()
    {
        warn!(target: "room_lobby", "discovery: room publish failed");
    }
    (room, params)
}

// needed helper: picks join params from the directory entry or recomputes them
fn room_params(
    namespace: &str,
    room: &str,
    params_override: Option<&str>,
    entry: Entry,
) -> Vec<u8> {
    if entry.params.is_empty() {
        resolve_params(namespace, room, params_override)
    } else {
        entry.params
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_params, room_params};
    use crate::discovery;

    #[test]
    fn test_usage() {
        let stored = discovery::directory::Entry {
            params: vec![7],
            peer_id: "peer".to_string(),
            addrs: Vec::new(),
            updated_at: 100,
        };
        assert_eq!(room_params("ns", "room", None, stored), vec![7]);
        let empty = discovery::directory::Entry {
            params: Vec::new(),
            peer_id: "peer".to_string(),
            addrs: Vec::new(),
            updated_at: 100,
        };
        assert_eq!(
            room_params("ns", "room", Some("ab"), empty),
            resolve_params("ns", "room", Some("ab"))
        );
    }
}
