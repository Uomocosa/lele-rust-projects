use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
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

    if args.iter().any(|a| a == "--connect") {
        run_client().await
    } else {
        run_host().await
    }
}

async fn run_host() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let bind_addr = args
        .iter()
        .position(|a| a == "--host")
        .and_then(|p| args.get(p + 1))
        .map(|s| {
            s.parse::<IpAddr>()
                .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
        })
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));

    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((bind_addr, 0))?;
    let ws_port = listener.local_addr()?.port();

    println!("Host node listening on {bind_addr}:{ws_port}");
    println!("Clients connect with: --connect {bind_addr}:{ws_port}");

    let ws_config = WebsocketApiConfig {
        address: bind_addr,
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let p2p_port = UdpSocket::bind("127.0.0.1:0")
        .and_then(|s| s.local_addr())
        .map(|a| a.port())
        .unwrap_or(31337);

    let config_args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: true,
            skip_load_from_network: true,
            public_address: Some(bind_addr),
            public_port: Some(p2p_port),
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
            tracing::error!(error = %e, "host node exited");
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

async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let connect_str = args
        .iter()
        .position(|a| a == "--connect")
        .and_then(|p| args.get(p + 1))
        .ok_or("--connect requires <host>:<port>")?;

    let parts: Vec<&str> = connect_str.split(':').collect();
    let host = parts.first().copied().unwrap_or("127.0.0.1");
    let port: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(7509);

    println!("Connecting to host node at {host}:{port}");

    let wasm = include_bytes!("../contract/clicker_contract.wasm").to_vec();
    let mut clicker = ClickerClient::connect(host, port, &wasm, Role::Subscribe).await?;
    println!("connected, counter state: {}", clicker.count());

    for i in 1..=10 {
        let count = clicker.tick().await?;
        println!("client tick {i}: count = {count}");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
