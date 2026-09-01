use std::net::{IpAddr, Ipv4Addr, TcpListener};

use freenet::config::{NetworkArgs, WebsocketApiConfig};
use freenet::local_node::NodeConfig;
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

use crate::testing;

/// Spawn a freenet node with the given `NetworkArgs` and return its WebSocket port.
///
/// # Errors
/// Returns an error if the listener or node build fails.
pub async fn spawn_with(
    tmp: &tempfile::TempDir,
    network: NetworkArgs,
) -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let port = listener.local_addr()?.port();
    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await?;
    let args = testing::node_args(tmp, network);
    let config = args.build().await?;
    let node_config = NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;
    tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            eprintln!("node exited with error: {e}");
        }
    });
    Ok(port)
}

// no test_usage necessary
