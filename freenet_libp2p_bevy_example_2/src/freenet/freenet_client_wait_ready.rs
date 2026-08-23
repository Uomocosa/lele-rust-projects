use std::time::Duration;

use freenet_stdlib::client_api::{
    ClientRequest, HostResponse, NodeDiagnosticsConfig, NodeQuery, QueryResponse,
};

use super::FreenetClient;
use crate::freenet;

pub async fn wait_ready(
    client: &mut FreenetClient,
    min_active_connections: usize,
    timeout: Duration,
) -> Result<(), freenet::FreenetConnectionError> {
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        client
            .send(ClientRequest::NodeQueries(NodeQuery::NodeDiagnostics {
                config: NodeDiagnosticsConfig::basic_status(),
            }))
            .await?;

        if let Some(Ok(HostResponse::QueryResponse(QueryResponse::NodeDiagnostics(diag)))) =
            client.recv_timeout(Duration::from_millis(500)).await
        {
            let active_connections = diag
                .network_info
                .as_ref()
                .map(|info| info.active_connections)
                .unwrap_or(0);
            if diag.node_info.is_some() && active_connections >= min_active_connections {
                return Ok(());
            }
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(freenet::FreenetConnectionError::ConnectionTimeout);
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

// no test_usage necessary — needs a live FreenetClient connection, exercised by the embedded-node integration tests
