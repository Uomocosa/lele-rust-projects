use crate::engine;
use crate::p2p;

#[derive(Debug)]
pub enum Command {
    Dial {
        peer_id: String,
        addrs: Vec<String>,
    },
    SendNetcode {
        peer_id: String,
        msg: p2p::NetcodeMsg,
    },
    RequestSnapshot {
        peer_id: String,
        player_id: engine::PlayerId,
    },
}
