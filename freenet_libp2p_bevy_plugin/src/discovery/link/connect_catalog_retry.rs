use tracing::warn;

use super::super::params::catalog_params::catalog_params;
use super::super::params::unique_game_id::UniqueGameId;
use super::catalog_client::CatalogClient;
use std::time::Duration;

pub async fn connect_catalog_retry(ws_port: u16, id: &UniqueGameId) -> CatalogClient {
    loop {
        match CatalogClient::connect(
            "127.0.0.1",
            ws_port,
            super::contract_wasm::contract_wasm(),
            &catalog_params(id),
        )
        .await
        {
            Ok(directory) => return directory,
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: catalog connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// no test_usage necessary
