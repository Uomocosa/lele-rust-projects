use std::time::Duration;

use tracing::{debug, warn};

use crate::discovery;
use discovery::freenet::basic::constants;

pub async fn connect_retry(
    host: &str,
    port: u16,
    params: &[u8],
    target: &tokio::sync::watch::Receiver<Option<discovery::session::PublishTarget>>,
) -> discovery::freenet::LobbyClient {
    let wasm = discovery::freenet::contract_wasm();
    let timeout = Duration::from_secs(constants::REQUEST_TIMEOUT_SECS);
    loop {
        let deploy = target.borrow().is_some();
        let attempt = discovery::freenet::connect(host, port, wasm, params, deploy);
        let Ok(result) = tokio::time::timeout(timeout, attempt).await else {
            warn!(target: "room_lobby", "discovery: lobby connect timed out, retrying");
            continue;
        };
        match result {
            Ok(lobby_client) => return lobby_client,
            Err(discovery::Error::ContractNotFound) => {
                debug!(target: "room_lobby", "discovery: no lobby yet and not in a room, waiting");
                tokio::time::sleep(Duration::from_secs(constants::MISSING_RETRY_SECS)).await;
            }
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: lobby connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(constants::CONNECT_RETRY_SECS)).await;
            }
        }
    }
}

// no test_usage necessary
