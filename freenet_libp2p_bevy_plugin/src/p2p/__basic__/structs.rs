use crate::net_id;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    pub peer_id: net_id::PeerId,
    pub addrs: Vec<net_id::PeerAddr>,
}
