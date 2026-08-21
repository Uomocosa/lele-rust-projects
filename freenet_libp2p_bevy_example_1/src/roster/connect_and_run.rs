use std::time::Duration;

use crate::roster;

/// Starts the embedded Freenet node, then runs the roster loop forever.
///
/// Node startup — bootstrap through the mainnet gateway index — can fail transiently: the
/// mainnet can refuse every gateway dial for minutes at a time (observed: `wait_ready`
/// timing out while all NAT traversals fail). A failed `start_embedded_node` is therefore
/// retried with capped exponential backoff instead of being terminal, mirroring
/// `connect_client_loop`'s setup retries. The game stays playable single-player throughout;
/// only the roster never joins until a retry succeeds.
pub async fn connect_and_run(
    args: roster::ConnectAndRunArgs,
    event_tx: tokio::sync::mpsc::UnboundedSender<roster::Event>,
) {
    let roster::ConnectAndRunArgs {
        local,
        gateway,
        contract_wasm,
        params,
        own_id,
        own_entry,
    } = args;

    let mut node_attempt: u32 = 0;
    let node = loop {
        node_attempt += 1;
        event_tx
            .send(roster::Event::Connecting {
                attempt: node_attempt,
            })
            .ok();
        let start_result = roster::start_embedded_node(local, gateway.clone()).await;
        match start_result {
            Ok(node) => break node,
            Err(e) => {
                let reason = e.to_string();
                event_tx
                    .send(roster::Event::ConnectionError(format!(
                        "failed to start embedded node (will retry): {reason}"
                    )))
                    .ok();
                let backoff = (roster::NODE_START_RETRY_BACKOFF_SECS * node_attempt as u64)
                    .min(roster::NODE_START_RETRY_MAX_BACKOFF_SECS);
                tracing::info!(
                    target: "roster",
                    attempt = node_attempt,
                    backoff,
                    "retrying embedded node startup"
                );
                tokio::time::sleep(Duration::from_secs(backoff)).await;
            }
        }
    };
    let roster::NodeInfo {
        host,
        ws_port,
        node_dir,
        ..
    } = node;
    let _node_dir = node_dir;

    let not_found_grace = if local {
        Duration::ZERO
    } else {
        Duration::from_secs(roster::SETUP_CONTRACT_GRACE_SECS)
    };

    roster::connect_client_loop(
        roster::ConnectClientArgs {
            host: &host,
            port: ws_port,
            contract_wasm: &contract_wasm,
            params: &params,
            own_id,
            own_entry,
            not_found_grace,
        },
        event_tx,
    )
    .await;
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
