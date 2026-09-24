use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use tracing::{info, warn};

use super::error::Error;

pub async fn bootstrap_node() -> Result<(tempfile::TempDir, u16), Error> {
    let tmp = tempfile::tempdir()?;
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener.local_addr().map_err(Error::from)?.port();
    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener)
        .await
        .map_err(|e| Error::Node(e.to_string()))?;
    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: false,
            network_port: Some(free_udp_port()?),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = args.build().await.map_err(|e| Error::Node(e.to_string()))?;
    let node_config = NodeConfig::new(config)
        .await
        .map_err(|e| Error::Node(e.to_string()))?;
    let node = node_config
        .build(clients)
        .await
        .map_err(|e| Error::Node(e.to_string()))?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            warn!(target: "room_lobby", error = %e, "embedded freenet node exited");
        }
    });
    info!(target: "room_lobby", ws_port, "embedded freenet node started");
    Ok((tmp, ws_port))
}

// needed helper: allocates a free UDP port so parallel instances never collide
fn free_udp_port() -> Result<u16, Error> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = socket.local_addr().map_err(Error::from)?.port();
    Ok(port)
}
// no test_usage necessary
