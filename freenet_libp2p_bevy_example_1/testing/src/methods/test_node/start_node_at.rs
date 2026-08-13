use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::path::Path;
use std::time::Duration;

use bevy_freenet::freenet::FreenetClient;
use freenet::ShutdownHandle;
use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use tokio::task::JoinHandle;

pub(crate) fn free_udp_port() -> Result<u16, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    Ok(socket.local_addr()?.port())
}

/// Shared body for `start_gateway()` and `start_peer()`: binds a fresh websocket port, builds
/// the embedded network node against `dir`, spawns it, and waits for the real readiness signal
/// (`FreenetClient::wait_ready`) instead of a blind sleep — a joining peer must wait until it
/// has actually connected to its gateway, not just until the node process is alive.
pub(crate) async fn start_node_at(
    dir: &Path,
    is_gateway: bool,
    public_port: u16,
    gateway: Option<String>,
) -> Result<(u16, String, JoinHandle<()>, ShutdownHandle), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,freenet::client_events::websocket=off".into()),
        )
        .try_init();

    let min_active_connections = usize::from(gateway.is_some());

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener.local_addr()?.port();

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;

    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway,
            skip_load_from_network: true,
            network_port: Some(public_port),
            public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(public_port),
            gateway: gateway.map(|g| vec![g]),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(dir.to_path_buf()),
            data_dir: Some(dir.to_path_buf()),
            log_dir: Some(dir.to_path_buf()),
        },
        ..Default::default()
    };
    let config = args.build().await?;
    let public_key_hex = hex::encode(config.transport_keypair().public().as_bytes());
    let node_config = NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;

    let shutdown_handle = node.shutdown_handle();

    let task = tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "node exited with error");
        }
    });

    let mut probe = FreenetClient::connect("127.0.0.1", ws_port).await?;
    probe
        .wait_ready(min_active_connections, Duration::from_secs(30))
        .await?;

    Ok((ws_port, public_key_hex, task, shutdown_handle))
}
