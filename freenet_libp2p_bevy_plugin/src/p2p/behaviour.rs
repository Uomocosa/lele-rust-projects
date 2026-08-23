use libp2p::swarm::NetworkBehaviour;
use libp2p::{autonat, dcutr, identify, ping, relay, request_response};

use crate::p2p;

#[derive(NetworkBehaviour)]
pub struct Behaviour<T: p2p::Message> {
    pub positions: request_response::Behaviour<p2p::MessageCodec<T>>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub autonat: autonat::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub relay: relay::client::Behaviour,
}
