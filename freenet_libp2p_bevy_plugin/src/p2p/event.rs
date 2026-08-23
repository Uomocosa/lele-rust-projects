use libp2p::PeerId;

use crate::p2p;

#[derive(Debug)]
pub enum Event<T> {
    Ready {
        peer_id: String,
        addrs: Vec<String>,
    },
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    IncomingSnapshot {
        from: PeerId,
        snapshot: p2p::Snapshot<T>,
    },
    Error(String),
}
