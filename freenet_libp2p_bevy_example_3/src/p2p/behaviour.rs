use libp2p::swarm::NetworkBehaviour;
use libp2p::{autonat, dcutr, identify, ping, relay, request_response};

use crate::p2p;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub netcode: request_response::Behaviour<p2p::NetcodeCodec>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub autonat: autonat::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub relay: relay::client::Behaviour,
}
