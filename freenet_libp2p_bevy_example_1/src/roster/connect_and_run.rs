use crate::boxes;
use crate::roster;

pub async fn connect_and_run(
    p2p_port: u16,
    local: bool,
    gateway: Option<String>,
    contract_wasm: Vec<u8>,
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
    event_tx: tokio::sync::mpsc::UnboundedSender<roster::Event>,
) {
    let node = match roster::start_embedded_node(p2p_port, local, gateway).await {
        Ok(node) => node,
        Err(e) => {
            event_tx
                .send(roster::Event::ConnectionError(format!(
                    "failed to start local node: {e}"
                )))
                .ok();
            return;
        }
    };
    let roster::NodeInfo {
        host,
        ws_port,
        node_dir,
        ..
    } = node;
    let _node_dir = node_dir;

    roster::connect_client_loop(
        &host,
        ws_port,
        &contract_wasm,
        &[],
        own_id,
        own_entry,
        event_tx,
    )
    .await;
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
