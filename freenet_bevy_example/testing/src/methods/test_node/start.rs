use std::net::{IpAddr, Ipv4Addr, TcpListener};
use std::path::Path;
use std::time::Duration;

use freenet::config::{ConfigArgs, ConfigPathsArgs, NetworkArgs, WebsocketApiConfig};
use freenet::local_node::{NodeConfig, OperationMode};
use freenet::run_network_node;
use freenet::server::serve_client_api_with_listener;
use freenet_stdlib::client_api::{ClientRequest, ContractRequest};
use freenet_stdlib::prelude::*;
use tokio::task::JoinHandle;

pub async fn start()
-> Result<crate::structs::test_node::TestNode, Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let (port, task) = start_node_at(tmp.path()).await?;

    Ok(crate::structs::test_node::TestNode {
        _tmp: Some(tmp),
        port,
        _task: task,
    })
}

/// Shared body for `start()` and `start_at()`: binds a fresh websocket port, builds the
/// embedded network node against `dir`, and spawns it. Retries a few times if the node's
/// on-disk store isn't ready yet — reusing a directory right after a prior node against it
/// shut down can transiently race the OS releasing that node's file lock.
pub(crate) async fn start_node_at(
    dir: &Path,
) -> Result<(u16, JoinHandle<()>), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt::try_init();

    let mut last_err: Option<Box<dyn std::error::Error>> = None;
    for attempt in 0..10u32 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(300 * attempt as u64)).await;
        }

        let listener = TcpListener::bind((IpAddr::V4(Ipv4Addr::LOCALHOST), 0))?;
        let port = listener.local_addr()?.port();

        let ws_config = WebsocketApiConfig {
            address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port,
            ..Default::default()
        };
        let clients = serve_client_api_with_listener(ws_config, listener).await?;

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
                config_dir: Some(dir.to_path_buf()),
                data_dir: Some(dir.to_path_buf()),
                log_dir: Some(dir.to_path_buf()),
            },
            ..Default::default()
        };
        let config = args.build().await?;
        let node_config = NodeConfig::new(config).await?;
        let node = match node_config.build(clients).await {
            Ok(node) => node,
            Err(e) => {
                last_err = Some(e.into());
                continue;
            }
        };
        let task = tokio::spawn(async move {
            if let Err(e) = run_network_node(node).await {
                tracing::error!(error = %e, "node exited with error");
            }
        });

        tokio::time::sleep(Duration::from_millis(500)).await;

        if probe_ready(port).await {
            tokio::time::sleep(Duration::from_millis(4500)).await;
            return Ok((port, task));
        }

        task.abort();
        let _ = task.await;
        last_err = Some("node store was not ready".into());
    }

    Err(last_err.unwrap_or_else(|| "node failed to start".into()))
}

/// Confirms the node's websocket API and contract store actually respond, not just that the
/// listener accepted a TCP connection.
async fn probe_ready(port: u16) -> bool {
    let Ok(client) =
        tokio::time::timeout(Duration::from_secs(2), crate::methods::connect::connect(port)).await
    else {
        return false;
    };
    let Ok(mut client) = client else {
        return false;
    };
    let probe_key = ContractKey::from_params_and_code(
        Parameters::from(Vec::new()),
        ContractCode::from(b"testing-probe".to_vec()),
    );
    let req = ContractRequest::Get {
        key: *probe_key.id(),
        return_contract_code: false,
        subscribe: false,
        blocking_subscribe: false,
    };
    if client.send(ClientRequest::ContractOp(req)).await.is_err() {
        return false;
    }
    matches!(
        tokio::time::timeout(Duration::from_secs(3), client.recv_response()).await,
        Ok(Ok(_))
    )
}

#[cfg(test)]
mod tests {
    use crate::structs::test_node::TestNode;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let node = TestNode::start().await.unwrap();
        assert!(node.port() > 0);
    }
}
