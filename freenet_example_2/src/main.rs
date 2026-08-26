use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use tracing::info;

use freenet_example_2::ClickerClient;
use freenet_example_2::Role;
use freenet_example_2::SetClient;

enum ContractMode {
    Counter,
    Set,
}

// needed helper:
fn contract_mode() -> ContractMode {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--contract-mode" {
            match args.next().as_deref() {
                Some("set") => return ContractMode::Set,
                _ => return ContractMode::Counter,
            }
        }
    }
    ContractMode::Counter
}

enum Connected {
    Counter(ClickerClient),
    Set(SetClient),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let has_role = args.iter().any(|a| a == "--role");

    let result = if has_role {
        run_client(contract_mode()).await
    } else {
        run_standalone(contract_mode()).await
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
    }
}

// needed helper:
async fn run_client(mode: ContractMode) -> Result<(), Box<dyn std::error::Error>> {
    let node_host = std::env::var("FREENET_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let node_port: u16 = std::env::var("FREENET_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7509);

    let mut connected = connect_with_retry(mode, &node_host, node_port).await?;
    run_loop(&mut connected).await
}

// needed helper:
async fn run_standalone(mode: ContractMode) -> Result<(), Box<dyn std::error::Error>> {
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
            is_gateway: !mainnet_client(),
            public_address: (!mainnet_client()).then_some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: (!mainnet_client()).then_some(p2p_port()),
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

    let mut connected = connect_with_retry(mode, "127.0.0.1", port).await?;
    run_loop(&mut connected).await
}

// needed helper:
// needed helper:
async fn connect_with_retry(
    mode: ContractMode,
    host: &str,
    port: u16,
) -> Result<Connected, Box<dyn std::error::Error>> {
    let mut attempt: u64 = 0;
    loop {
        attempt += 1;
        let result = match &mode {
            ContractMode::Counter => {
                let wasm = include_bytes!("../contract/clicker_contract.wasm");
                ClickerClient::connect_with_params(
                    host,
                    port,
                    wasm,
                    &contract_params(),
                    Role::Publish,
                )
                .await
                .map(Connected::Counter)
            }
            ContractMode::Set => {
                let wasm = include_bytes!("../contract/set_contract.wasm");
                SetClient::connect(host, port, wasm, &contract_params(), instance_tag())
                    .await
                    .map(Connected::Set)
            }
        };
        match result {
            Ok(c) => return Ok(c),
            Err(e) => {
                tracing::warn!(exception = %e, attempt, "connect failed, retrying");
                let backoff = std::cmp::min(attempt * 3, 30);
                tokio::time::sleep(Duration::from_secs(backoff)).await;
            }
        }
    }
}

// needed helper:
// needed helper:
async fn run_loop(connected: &mut Connected) -> Result<(), Box<dyn std::error::Error>> {
    match connected {
        Connected::Counter(c) => {
            info!(
                mainnet = mainnet_client(),
                key = %c.contract_key(),
                count = c.count(),
                "connected, running indefinitely"
            );
            loop {
                match c.tick().await {
                    Ok(count) => info!(count, "tick"),
                    Err(e) => eprintln!("tick error: {e}"),
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        Connected::Set(s) => {
            info!(
                mainnet = mainnet_client(),
                tag = s.tag,
                key = %s.contract_key(),
                count = s.count(),
                "connected, running indefinitely"
            );
            loop {
                match s.tick().await {
                    Ok(count) => info!(count, owns = s.own_count(), "tick"),
                    Err(e) => eprintln!("tick error: {e}"),
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

// needed helper:
// needed helper:
fn instance_tag() -> u64 {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--instance-tag"
            && let Some(val) = args.next()
            && let Ok(v) = val.parse::<u64>()
        {
            return v;
        }
    }
    0
}

// needed helper:
// needed helper:
fn mainnet_client() -> bool {
    std::env::args().any(|a| a == "--mainnet-client")
}

// needed helper:
// needed helper:
fn contract_params() -> Vec<u8> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--contract-params"
            && let Some(val) = args.next()
            && let Ok(bytes) = hex::decode(val.trim_start_matches("0x"))
        {
            return bytes;
        }
    }
    Vec::new()
}

// needed helper:
// needed helper:
fn p2p_port() -> u16 {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--p2p-port"
            && let Some(val) = args.next()
            && let Ok(port) = val.parse::<u16>()
        {
            return port;
        }
    }
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .expect("failed to probe an available UDP port");
    socket
        .local_addr()
        .expect("failed to read assigned port")
        .port()
}

#[cfg(test)]
mod tests {
    use super::{contract_mode, instance_tag};

    #[test]
    fn test_usage() {
        let _ = contract_mode();
        let _ = instance_tag();
    }
}
