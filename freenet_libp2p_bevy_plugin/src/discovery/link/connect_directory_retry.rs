use tracing::warn;

use super::super::params::dir_params::dir_params;
use super::directory_client::DirectoryClient;
use std::time::Duration;

pub async fn connect_directory_retry(ws_port: u16, namespace: &str) -> DirectoryClient {
    loop {
        match DirectoryClient::connect(
            "127.0.0.1",
            ws_port,
            super::contract_wasm::contract_wasm(),
            &dir_params(namespace),
        )
        .await
        {
            Ok(directory) => return directory,
            Err(e) => {
                warn!(target: "room_lobby", error = %e, "discovery: directory connect failed, retrying");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

// no test_usage necessary
