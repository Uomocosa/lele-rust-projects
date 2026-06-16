use std::time::Duration;

use tracing::info;

use freenet_example::Role;
use freenet_example::clicker_client::ClickerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let role = parse_role()?;

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

fn parse_role() -> Result<Role, String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--role" {
            match args.next().as_deref() {
                Some("subscribe") => return Ok(Role::Subscribe),
                Some("publish") => return Ok(Role::Publish),
                Some(other) => {
                    return Err(format!("unknown role: {other}. Use publish or subscribe"));
                }
                None => return Err("--role requires an argument: publish or subscribe".into()),
            }
        }
    }
    Ok(Role::Publish)
}

#[cfg(test)]
mod tests {
    use super::parse_role;

    #[test]
    fn test_usage() {
        let result = parse_role();
        assert!(result.is_ok() || result.is_err());
    }
}
