use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use freenet_example::ClickerClient;
use freenet_example::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--gateway") {
        run_gateway().await
    } else if let Some(pos) = args.iter().position(|a| a == "--connect") {
        let connect_str = args
            .get(pos + 1)
            .ok_or("--connect requires <ip>:<port>,<pubkey>")?;
        run_peer(connect_str).await
    } else {
        eprintln!("Usage:");
        eprintln!(
            "  Gateway: cargo run --example p2p_counter_gateway -- --gateway --public-address <YOUR_IP>"
        );
        eprintln!(
            "  Peer:    cargo run --example p2p_counter_gateway -- --connect <GATEWAY_IP>:<PORT>,<PUBKEY>"
        );
        std::process::exit(1);
    }
}

async fn run_gateway() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let public_addr = args
        .iter()
        .position(|a| a == "--public-address")
        .and_then(|p| args.get(p + 1))
        .ok_or("--gateway requires --public-address <IP>")?;
    let public_ip: IpAddr = public_addr.parse()?;

    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener.local_addr()?.port();

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: true,
            skip_load_from_network: true,
            public_address: Some(public_ip),
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
    let secrets = tmp.path().join("secrets");
    tracing::info!(gateway_addr = %public_ip, ws_port = %ws_port, secrets = %secrets.display(), "gateway started");
    println!("Gateway started at {public_addr}");
    println!("Secrets directory: {}", secrets.display());
    println!("Share this with peers: --connect {public_addr}:31337,<pubkey-from-secrets>");

    let node = node_config.build(clients).await?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "gateway exited");
        }
    });

    tokio::time::sleep(Duration::from_secs(3)).await;

    let wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
    let mut clicker = ClickerClient::connect("127.0.0.1", ws_port, &wasm, Role::Publish).await?;
    println!("counter deployed, initial count: {}", clicker.count());

    for i in 1..=10 {
        let count = clicker.tick().await?;
        println!("tick {i}: count = {count}");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

async fn run_peer(connect_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener.local_addr()?.port();

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            skip_load_from_network: true,
            gateway: Some(vec![connect_str.to_string()]),
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
            tracing::error!(error = %e, "peer node exited");
        }
    });

    tokio::time::sleep(Duration::from_secs(3)).await;

    let wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
    let mut clicker = ClickerClient::connect("127.0.0.1", ws_port, &wasm, Role::Subscribe).await?;
    println!("connected to gateway, counter state: {}", clicker.count());

    for i in 1..=10 {
        let count = clicker.tick().await?;
        println!("peer tick {i}: count = {count}");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
