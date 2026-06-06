//! Minimal freenet client — connects to a local freenet node via WebSocket,
//! deploys the clicker counter contract, increments it, and subscribes to updates.
//!
//! Prerequisites:
//!   1. Build the contract:
//!        cd contract && cargo build --release --target wasm32-unknown-unknown
//!   2. Start a local freenet node (e.g. `freenet --local`)
//!   3. Run this binary

use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load compiled contract WASM
    let _contract_wasm = std::fs::read(
        "contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm",
    )?;

    // TODO: Connect to local freenet WebSocket API (default ws://127.0.0.1:5050)
    // and deploy the contract, get/subscribe/update its state.
    //
    // The freenet WebSocket API uses flatbuffers for message encoding.
    // See freenet's `client_api` module and WebSocket example for details.
    //
    // Pseudo-code:
    //   let client = FreenetClient::connect("ws://127.0.0.1:5050").await?;
    //   let key = client.deploy_contract(contract_wasm).await?;
    //   let state: u64 = client.get_state(&key).await?;
    //   info!("Initial count: {state}");
    //   for i in 0..5 {
    //       client.update_contract(&key, &bincode::serialize(&(i+1))?).await?;
    //       tokio::time::sleep(Duration::from_secs(1)).await;
    //   }

    info!("Client ready — connect to a local freenet node to use this example");
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(())
}
