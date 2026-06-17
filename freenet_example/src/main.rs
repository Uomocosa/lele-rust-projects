use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use tracing::info;

use freenet_example::Role;
use freenet_example::clicker_client::ClickerClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let has_role = args.iter().any(|a| a == "--role");

    let result = if has_role {
        run_client().await
    } else {
        run_standalone().await
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let role = parse_role()?;

    let contract_wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
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

async fn run_standalone() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();

    info!(port, "starting in-process network-mode node");

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: true,
            skip_load_from_network: true,
            public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(31337),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = config_args.build().await?;
    let node_config = NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;

    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "node exited with error");
        }
    });

    tokio::time::sleep(Duration::from_secs(3)).await;

    let contract_wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
    let mut clicker = ClickerClient::connect("127.0.0.1", port, &contract_wasm, Role::Publish).await?;

    info!(key = %clicker.contract_key(), count = clicker.count(), "connected, running indefinitely");

    loop {
        match clicker.tick().await {
            Ok(count) => info!(count, "tick"),
            Err(e) => eprintln!("tick error: {e}"),
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn parse_role() -> Result<Role, String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--standalone" => {}
            "--role" => match args.next().as_deref() {
                Some("subscribe") => return Ok(Role::Subscribe),
                Some("publish") => return Ok(Role::Publish),
                Some(other) => {
                    return Err(format!("unknown role: {other}. Use publish or subscribe"));
                }
                None => return Err("--role requires an argument: publish or subscribe".into()),
            },
            _ => {}
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
