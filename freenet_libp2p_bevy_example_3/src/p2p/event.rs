use libp2p::PeerId;

use crate::p2p;

#[derive(Debug)]
pub enum Event {
    Ready {
        peer_id: String,
        addrs: Vec<String>,
    },
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    IncomingSnapshot {
        from: PeerId,
        snapshot: p2p::Snapshot,
    },
    IncomingNetcode {
        from: PeerId,
        msg: p2p::NetcodeMsg,
    },
    Error(String),
}
