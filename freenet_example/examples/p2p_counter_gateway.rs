use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use freenet::transport::TransportKeypair;

use freenet_example::GlobalCounterClient;
use freenet_example::Role;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let is_gateway = args.iter().any(|a| a == "--gateway");
    let connect_pos = args.iter().position(|a| a == "--connect");

    if is_gateway {
        let connect_str =
            connect_pos.and_then(|p| p.checked_add(1).and_then(|i| args.get(i).cloned()));
        run_gateway(connect_str).await
    } else if let Some(pos) = connect_pos {
        let connect_str = pos
            .checked_add(1)
            .and_then(|i| args.get(i))
            .ok_or("--connect requires <ip>:<port>,<pubkey>")?;
        run_peer(connect_str).await
    } else {
        eprintln!("Usage:");
        eprintln!(
            "  Gateway: cargo run --example p2p_counter_gateway -- --gateway --public-address <IP>"
        );
        eprintln!(
            "  Gateway+peer: cargo run --example p2p_counter_gateway -- --gateway --public-address <IP> --connect <GATEWAY>:<PORT>,<PUBKEY>"
        );
        eprintln!(
            "  Peer:    cargo run --example p2p_counter_gateway -- --connect <GATEWAY>:<PORT>,<PUBKEY>"
        );
        std::process::exit(1);
    }
}

async fn run_gateway(upstream_gateway: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let public_addr = args
        .iter()
        .position(|a| a == "--public-address")
        .and_then(|p| p.checked_add(1).and_then(|i| args.get(i)))
        .ok_or("--gateway requires --public-address <IP>")?;
    let public_ip: IpAddr = public_addr.parse()?;

    let p2p_port = UdpSocket::bind("127.0.0.1:0")
        .and_then(|s| s.local_addr())
        .map_or(31337, |a| a.port());

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
            public_port: Some(p2p_port),
            gateway: upstream_gateway.map(|s| vec![s]),
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

    let keypair = TransportKeypair::load(tmp.path().join("secrets").join("transport_keypair"))?;
    let pubkey_hex = hex::encode(keypair.public().as_bytes());
    println!("GATEWAY_CONNECT=127.0.0.1:{p2p_port},{pubkey_hex}");

    let node = node_config.build(clients).await?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "gateway exited");
        }
    });

    tokio::time::sleep(Duration::from_secs(5)).await;

    let wasm = include_bytes!("../contract/global_counter_contract.wasm").to_vec();
    let mut global_counter =
        GlobalCounterClient::connect("127.0.0.1", ws_port, &wasm, Role::Publish).await?;
    println!(
        "counter deployed, initial count: {}",
        global_counter.count()
    );

    for i in 1..=10 {
        let count = global_counter.tick().await?;
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

    tokio::time::sleep(Duration::from_secs(5)).await;

    let wasm = include_bytes!("../contract/global_counter_contract.wasm").to_vec();
    let mut global_counter =
        GlobalCounterClient::connect("127.0.0.1", ws_port, &wasm, Role::Subscribe).await?;
    println!(
        "connected to gateway, counter state: {}",
        global_counter.count()
    );

    for i in 1..=10 {
        let count = global_counter.tick().await?;
        println!("peer tick {i}: count = {count}");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
