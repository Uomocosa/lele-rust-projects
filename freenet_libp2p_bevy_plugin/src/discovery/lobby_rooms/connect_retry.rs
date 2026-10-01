use std::time::Duration;

use tracing::{debug, warn};

use crate::discovery;
use discovery::Error;
use discovery::basic::constants;
use discovery::id::UniqueGameId;
use discovery::lobby_rooms::IndexClient;
use discovery::lobby_rooms::board_params::board_params;
use discovery::lobby_rooms::connect::connect;
use discovery::lobby_rooms::contract_wasm::contract_wasm;
use discovery::session::BoardTarget;

pub async fn connect_retry(
    host: &str,
    port: u16,
    id: &UniqueGameId,
    target: &tokio::sync::watch::Receiver<Option<BoardTarget>>,
) -> IndexClient {
    loop {
        let deploy = target.borrow().is_some();
        let params = board_params(id);
        let attempt = connect(host, port, contract_wasm(), &params, deploy);
        let timeout = Duration::from_secs(constants::BOARD_REQUEST_TIMEOUT_SECS);
        let Ok(result) = tokio::time::timeout(timeout, attempt).await else {
            warn!(target: "room_lobby", "discovery: board connect timed out, retrying");
            continue;
        };
        match result {
            Ok(client) => return client,
            Err(Error::ContractNotFound) => {
                debug!(target: "room_lobby", "discovery: no board yet and not in a room, waiting");
                tokio::time::sleep(Duration::from_secs(constants::BOARD_MISSING_RETRY_SECS)).await;
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: board connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(constants::CONNECT_RETRY_SECS)).await;
            }
        }
    }
}

// no test_usage necessary
