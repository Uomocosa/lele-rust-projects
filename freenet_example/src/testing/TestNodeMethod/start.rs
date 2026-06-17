use std::net::{IpAddr, Ipv4Addr, TcpListener};

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;

pub async fn start() -> crate::testing::test_node::TestNode {
    let _ = tracing_subscriber::fmt::try_init();

    let tmp = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0)).unwrap();
    let port = listener.local_addr().unwrap().port();

    let ws_config = WebsocketApiConfig {
        address: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port,
        ..Default::default()
    };
    let clients = serve_client_api_with_listener(ws_config, listener).await.unwrap();

    let args = ConfigArgs {
        mode: Some(OperationMode::Network),
        network_api: NetworkArgs {
            is_gateway: true,
            skip_load_from_network: true,
            public_address: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            public_port: Some(31337),
            ..Default::default()
        },
        config_paths: ConfigPathsArgs {
            config_dir: Some(tmp.path().to_path_buf()),
            data_dir: Some(tmp.path().to_path_buf()),
            log_dir: Some(tmp.path().to_path_buf()),
        },
        ..Default::default()
    };
    let config = args.build().await.unwrap();
    let node_config = NodeConfig::new(config).await.unwrap();
    let node = node_config.build(clients).await.unwrap();
    let task = tokio::spawn(async move {
        if let Err(e) = run_network_node(node).await {
            tracing::error!(error = %e, "node exited with error");
        }
    });

    crate::testing::test_node::TestNode {
        _tmp: tmp,
        port,
        _task: task,
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::TestNode;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await;
        assert!(node.port() > 0);
    }
}
