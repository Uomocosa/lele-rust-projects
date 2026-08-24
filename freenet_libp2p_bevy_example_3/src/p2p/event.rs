use libp2p::PeerId;

use crate::p2p;

#[derive(Debug)]
pub enum Event {
    Ready { peer_id: String, addrs: Vec<String> },
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    IncomingNetcode { from: PeerId, msg: p2p::NetcodeMsg },
    Error(String),
}
