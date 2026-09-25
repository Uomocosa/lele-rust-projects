use std::time::Duration;

use tracing::warn;

use crate::discovery;
use discovery::basic::constants;
use discovery::id::UniqueGameId;
use discovery::lobby_rooms::IndexClient;
use discovery::lobby_rooms::board_params::board_params;
use discovery::lobby_rooms::connect::connect;
use discovery::lobby_rooms::contract_wasm::contract_wasm;

pub async fn connect_retry(host: &str, port: u16, id: &UniqueGameId) -> IndexClient {
    loop {
        match connect(host, port, contract_wasm(), &board_params(id)).await {
            Ok(client) => return client,
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: board connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(constants::CONNECT_RETRY_SECS)).await;
            }
        }
    }
}

// no test_usage necessary
