use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use freenet_example_2::ClickerClient;
use freenet_example_2::Role;

fn detect_public_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?;
    (!local.ip().is_loopback()).then_some(local.ip())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener.local_addr()?.port();

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let public_ip = detect_public_ip().unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    println!("detected public IP: {public_ip}");

    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            skip_load_from_network: false,
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
    let config = args.build().await?;
    let node_config = NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "node exited");
        }
    });

    tokio::time::sleep(Duration::from_secs(5)).await;

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
