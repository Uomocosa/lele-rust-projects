use std::time::Duration;

use tracing::warn;

use super::super::constants;
use super::super::id::UniqueGameId;
use super::board_params::board_params;
use super::connect::connect;
use super::contract_wasm::contract_wasm;
use super::index_client::IndexClient;

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
