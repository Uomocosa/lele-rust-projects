use std::net::{IpAddr, Ipv4Addr, TcpListener, UdpSocket};
use std::time::Duration;

use tracing::info;

use crate::freenet;
use crate::roster;

// needed helper:
fn free_udp_port() -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
    let socket = UdpSocket::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    Ok(socket.local_addr()?.port())
}

/// Starts an in-process network-mode Freenet node and returns its dial-in info.
pub async fn start_embedded_node(
    local: bool,
    gateway: Option<String>,
) -> Result<roster::NodeInfo, Box<dyn std::error::Error + Send + Sync>> {
    let tmp = tempfile::tempdir()?;

    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
    let ws_port = listener.local_addr()?.port();
    let public_port = free_udp_port()?;
    let skip_load_from_network = local || gateway.is_some();
    let is_gateway = local;
    let public_address = local.then_some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    let min_active_connections = if local { 0 } else { 1 };

    info!(
        target: "roster",
        ws_port,
        public_port,
        skip_load_from_network,
        is_gateway,
        ?gateway,
        "starting in-process network-mode node"
    );

    let ws_config = ::freenet::config::WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: ws_port,
        ..Default::default()
    };
    let clients = ::freenet::server::serve_client_api_with_listener(ws_config, listener).await?;

    let config_args = ::freenet::config::ConfigArgs {
        mode: Some(::freenet::local_node::OperationMode::Network),
        network_api: ::freenet::config::NetworkArgs {
            is_gateway,
            skip_load_from_network,
            network_port: Some(public_port),
            public_address,
            public_port: public_address.map(|_| public_port),
            gateway: gateway.map(|g| vec![g]),
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
    let public_key_hex = hex::encode(config.transport_keypair().public().as_bytes());
    let node_config = ::freenet::local_node::NodeConfig::new(config).await?;
    let node = node_config.build(clients).await?;

    let node_task = tokio::spawn(async move {
        if let Err(e) = ::freenet::run_network_node(node).await {
            tracing::error!(target: "roster", error = %e, "node exited with error");
        }
    });

    let ready_result: Result<(), Box<dyn std::error::Error + Send + Sync>> = async {
        let mut probe = freenet::FreenetClient::connect("127.0.0.1", ws_port).await?;
        probe
            .wait_ready(min_active_connections, Duration::from_secs(90))
            .await?;
        Ok(())
    }
    .await;
    if let Err(e) = ready_result {
        node_task.abort();
        tracing::error!(
            target: "roster",
            error = %e,
            public_port,
            "embedded node failed to become ready; aborted node task and releasing port"
        );
        return Err(e);
    }

    info!(
        target: "roster",
        public_key_hex,
        public_port,
        "embedded node ready; dial as 127.0.0.1:<public-port>,<pubkey-hex>"
    );

    Ok(roster::NodeInfo {
        host: "127.0.0.1".to_string(),
        ws_port,
        public_port,
        public_key_hex,
        node_dir: tmp,
    })
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
