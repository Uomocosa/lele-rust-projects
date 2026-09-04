use libp2p::kad::store::MemoryStore;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{identify, kad, ping, request_response};

use crate::p2p;

#[derive(NetworkBehaviour)]
pub struct Behaviour<T: p2p::Message> {
    pub request_response: request_response::Behaviour<p2p::MessageCodec<T>>,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub identify: identify::Behaviour,
    pub ping: ping::Behaviour,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_usage() {
        assert!(true);
    }
}
// no test_usage necessary
