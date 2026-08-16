use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::server::serve_client_api_with_listener;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

fn free_udp_port() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    Ok(socket.local_addr()?.port())
}

/// Starts an in-process freenet **network-mode** node that binds an inbound UDP port, then
/// connects to its local websocket API and confirms the node reached readiness. The whole point
/// is to exercise the exact inbound-listen path that triggers the Windows Firewall "Query User"
/// prompt: if NotifyOnListen is False (or a broad allow rule exists) the bind proceeds silently
/// and this completes; if a prompt were to appear/block, the job timeout turns it into a failure.
///
/// Bevy-free on purpose: this crate depends only on `freenet` so the probe builds in ~2-3 min
/// instead of compiling the whole bevy dependency tree.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,freenet::client_events::websocket=off".into()),
        )
        .try_init();

    let tmp = tempfile::tempdir()?;
    let public_port = free_udp_port()?;

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
            is_gateway: true,
            skip_load_from_network: true,
            network_port: Some(public_port),
            public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(public_port),
            gateway: None,
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

    let shutdown = node.shutdown_handle();
    tokio::spawn(async move {
        if let Err(e) = freenet::run_network_node(node).await {
            tracing::error!(error = %e, "node exited with error");
        }
    });

    let url = format!("ws://127.0.0.1:{ws_port}/v1/contract/command?encodingProtocol=native");
    let mut request = url
        .into_client_request()
        .map_err(|e| format!("build ws request failed: {e}"))?;
    request.headers_mut().insert(
        "encoding-protocol",
        http::HeaderValue::from_static("native"),
    );
    let (mut ws, _resp) = tokio::time::timeout(
        Duration::from_secs(10),
        tokio_tungstenite::connect_async(request),
    )
    .await
    .map_err(|_| "connect to node ws timed out")?
    .map_err(|e| format!("connect to node ws failed: {e}"))?;
    println!("firewall-probe: node bound inbound UDP :{public_port}, ws :{ws_port}");
    tokio::time::sleep(Duration::from_secs(3)).await;
    ws.close(None).await.ok();
    shutdown.shutdown().await;
    println!("firewall-probe: PASS (node started, inbound UDP bound, no prompt hang)");
    Ok(())
}
