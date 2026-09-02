use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::sync::OnceLock;
use std::time::Duration;

use clap::{Parser, ValueEnum};
use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use tracing::info;

use freenet_example::GlobalCounterClient;
use freenet_example::Role;
use freenet_example::SetClient;

#[derive(Parser, Debug)]
#[command(name = "freenet-example-3")]
struct Args {
    #[arg(long, value_enum, default_value = "counter")]
    contract_mode: ContractModeArg,
    #[arg(long)]
    role: Option<String>,
    #[arg(long, default_value_t = 0)]
    instance_tag: u64,
    #[arg(long)]
    contract_params: Option<String>,
    #[arg(long)]
    p2p_port: Option<u16>,
    #[arg(long, default_value_t = false)]
    mainnet_client: bool,
    #[arg(long, default_value_t = false)]
    standalone: bool,
}

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum ContractModeArg {
    Counter,
    Set,
}

enum ContractMode {
    Counter,
    Set,
}

// needed helper:
fn args() -> &'static Args {
    static CELL: OnceLock<Args> = OnceLock::new();
    CELL.get_or_init(Args::parse)
}

// needed helper:
fn contract_mode() -> ContractMode {
    match args().contract_mode {
        ContractModeArg::Set => ContractMode::Set,
        ContractModeArg::Counter => ContractMode::Counter,
    }
}

enum Connected {
    Counter(GlobalCounterClient),
    Set(SetClient),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout)
        .init();

    let has_role = args().role.is_some();
    let mode = contract_mode();

    let result = if has_role {
        run_client(mode).await
    } else {
        run_standalone(mode).await
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
            public_address: (!mainnet_client()).then_some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
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
        attempt = attempt.saturating_add(1);
        let result = match &mode {
            ContractMode::Counter => {
                let wasm = include_bytes!("../contract/global_counter_contract.wasm");
                GlobalCounterClient::connect_with_tag(
                    host,
                    port,
                    wasm,
                    &contract_params(),
                    Role::Publish,
                    instance_tag(),
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
                let backoff = std::cmp::min(attempt.saturating_mul(3), 30);
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
                tag = c.tag,
                key = %c.contract_key,
                count = c.count(),
                owns = c.own(),
                "connected, running indefinitely"
            );
            loop {
                match c.tick().await {
                    Ok(count) => info!(count, owns = c.own(), "tick"),
                    Err(e) => eprintln!("tick error: {e}"),
                }
                c.note_foreign_slots();
                if let Err(e) = c.bridge_tick().await {
                    eprintln!("bridge error: {e}");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        Connected::Set(s) => {
            info!(
                mainnet = mainnet_client(),
                tag = s.tag,
                key = %s.contract_key,
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
fn instance_tag() -> u64 {
    args().instance_tag
}

// needed helper:
fn mainnet_client() -> bool {
    args().mainnet_client
}

// needed helper:
fn contract_params() -> Vec<u8> {
    args()
        .contract_params
        .as_deref()
        .and_then(|v| hex::decode(v.trim_start_matches("0x")).ok())
        .unwrap_or_default()
}

fn p2p_port() -> u16 {
    if let Some(port) = args().p2p_port {
        return port;
    }
    UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .and_then(|s| s.local_addr())
        .map_or(0, |a| a.port())
}

#[cfg(test)]
mod tests {
    use super::ContractModeArg;

    #[test]
    fn test_usage() {
        let _ = ContractModeArg::Counter;
        let _ = ContractModeArg::Set;
    }
}
