use std::time::{Duration, Instant};

use tracing::{info, warn};

use super::super::params::player_id::PlayerId;
use super::roster_client::RosterClient;

pub async fn connect_roster_retry(
    ws_port: u16,
    room: &str,
    room_params: &[u8],
    own: PlayerId,
    peer_id: &str,
    addrs: &[String],
) -> RosterClient {
    loop {
        let attempt = Instant::now();
        match RosterClient::connect(
            "127.0.0.1",
            ws_port,
            super::contract_wasm::contract_wasm(),
            room_params,
            own,
            peer_id,
            addrs,
        )
        .await
        {
            Ok(roster) => {
                info!(target: "room_lobby", room = %room, slots = roster.slots.len(), elapsed_ms = attempt.elapsed().as_millis(), "discovery: roster fetched");
                return roster;
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: roster connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// no test_usage necessary
