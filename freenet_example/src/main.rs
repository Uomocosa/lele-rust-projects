use std::time::Duration;

use tracing::info;

use freenet_example::clicker::{ClickerClient, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let role = parse_role();

    let contract_wasm =
        std::fs::read("contract/target/wasm32-unknown-unknown/release/clicker_contract.wasm")?;
    info!(target: "freenet_example", size = contract_wasm.len(), "loaded contract wasm");

    let node_host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let node_port: u16 = std::env::var("FREENET_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7509);

    let mut clicker = ClickerClient::connect(&node_host, node_port, &contract_wasm, role).await?;
    info!(target: "freenet_example", key = %clicker.contract_key(), count = clicker.count(), "connected and subscribed");

    loop {
        let count = clicker.tick().await?;
        info!(target: "freenet_example", count, "tick done");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn parse_role() -> Role {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--role" {
            match args.next().as_deref() {
                Some("subscribe") => return Role::Subscribe,
                Some("publish") => return Role::Publish,
                Some(other) => {
                    eprintln!("unknown role: {other}. Use publish or subscribe");
                    std::process::exit(1);
                }
                None => {
                    eprintln!("--role requires an argument: publish or subscribe");
                    std::process::exit(1);
                }
            }
        }
    }
    Role::Publish
}
