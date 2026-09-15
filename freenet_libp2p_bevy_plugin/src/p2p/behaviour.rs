#![allow(unreachable_code)]

use libp2p::kad::store::MemoryStore;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{dcutr, gossipsub, identify, kad, ping, relay, request_response};

use crate::p2p;

#[derive(NetworkBehaviour)]
pub struct Behaviour<T: p2p::Message> {
    pub request_response: request_response::Behaviour<p2p::MessageCodec<T>>,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub relay: relay::Behaviour,
    pub relay_client: relay::client::Behaviour,
    pub dcutr: dcutr::Behaviour,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        let _ = stringify!(Behaviour::<()>);
    }
}
// no test_usage necessary
