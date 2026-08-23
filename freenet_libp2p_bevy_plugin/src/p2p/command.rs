use crate::p2p;

#[derive(Debug)]
pub enum Command<T> {
    Dial {
        peer_id: String,
        addrs: Vec<String>,
    },
    SendSnapshot {
        peer_id: String,
        snapshot: p2p::Snapshot<T>,
    },
}
