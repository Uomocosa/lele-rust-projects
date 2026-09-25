use crate::p2p;

use tracing::info;

pub fn dial_addrs(
    net_tx: &tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
    peer_id: &str,
    addrs: &[String],
    kad_addrs: &[String],
) {
    if addrs.is_empty() {
        return;
    }
    info!(target: "room_lobby", peer = %peer_id, addrs = ?addrs, "discovery: dialing peer");
    net_tx
        .send(p2p::NetCommand::Dial {
            peer_id: peer_id.to_string(),
            addrs: addrs.to_vec(),
        })
        .ok();
    net_tx
        .send(p2p::NetCommand::AddKadPeer {
            peer_id: peer_id.to_string(),
            addrs: kad_addrs.to_vec(),
        })
        .ok();
}

// no test_usage necessary
