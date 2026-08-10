use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::time::Duration;

use tracing::info;

use crate::freenet;

// needed helper:
/// Returns the `TempDir` so the caller can keep it alive: it backs the node's config, data and
/// log dirs, and dropping it deletes them out from under the still-running node.
pub async fn start_embedded_node(
    p2p_port: u16,
) -> Result<(String, u16, tempfile::TempDir), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();

    info!(target: "roster", port, "starting in-process network-mode node");

    let ws_config = ::freenet::config::WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = ::freenet::server::serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ::freenet::config::ConfigArgs {
        mode: Some(::freenet::local_node::OperationMode::Network),
        network_api: ::freenet::config::NetworkArgs {
            is_gateway: true,
            network_port: Some(p2p_port),
            public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(p2p_port),
            ..Default::default()
        },
        config_paths: ::freenet::config::ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = config_args.build().await?;
    let node_config = ::freenet::local_node::NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;

    tokio::spawn(async move {
        if let Err(e) = ::freenet::run_network_node(node).await {
            tracing::error!(target: "roster", error = %e, "node exited with error");
        }
    });

    let mut probe = freenet::FreenetClient::connect("127.0.0.1", port).await?;
    probe.wait_ready(0, Duration::from_secs(30)).await?;

    Ok(("127.0.0.1".to_string(), port, tmp))
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
