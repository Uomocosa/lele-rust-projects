use crate::boxes;
use crate::roster;

pub async fn connect_and_run(
    p2p_port: u16,
    contract_wasm: Vec<u8>,
    own_id: boxes::PlayerId,
    own_entry: roster::PeerEntry,
    event_tx: tokio::sync::mpsc::UnboundedSender<roster::Event>,
) {
    let _node_dir;
    let (host, port) = match roster::start_embedded_node(p2p_port).await {
        Ok((host, port, dir)) => {
            _node_dir = dir;
            (host, port)
        }
        Err(e) => {
            event_tx
                .send(roster::Event::ConnectionError(format!(
                    "failed to start local node: {e}"
                )))
                .ok();
            return;
        }
    };

    roster::connect_client_loop(
        &host,
        port,
        &contract_wasm,
        &[],
        own_id,
        own_entry,
        event_tx,
    )
    .await;
}
// no test_usage necessary — needs a live embedded freenet node, exercised by testing/
