use std::time::Duration;

use libp2p::identity::Keypair;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::{StreamProtocol, autonat, dcutr, identify, ping, relay};

use super::behaviour::Behaviour;
use crate::p2p;

pub fn new(key: &Keypair, relay_behaviour: relay::client::Behaviour) -> Behaviour {
    let peer_id = key.public().to_peer_id();
    Behaviour {
        netcode: request_response::Behaviour::with_codec(
            p2p::NetcodeCodec,
            [(
                StreamProtocol::new(p2p::constants::NETCODE_PROTOCOL_NAME),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        ),
        identify: identify::Behaviour::new(identify::Config::new(
            p2p::constants::IDENTIFY_PROTOCOL_VERSION.to_string(),
            key.public(),
        )),
        ping: ping::Behaviour::new(
            ping::Config::new()
                .with_interval(Duration::from_secs(p2p::constants::PING_INTERVAL_SECS)),
        ),
        autonat: autonat::Behaviour::new(peer_id, autonat::Config::default()),
        dcutr: dcutr::Behaviour::new(peer_id),
        relay: relay_behaviour,
    }
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;
    use libp2p::relay;

    use super::new;

    #[test]
    fn test_usage() {
        let key = Keypair::generate_ed25519();
        let peer_id = key.public().to_peer_id();
        let (_, relay_behaviour) = relay::client::new(peer_id);

        let behaviour = new(&key, relay_behaviour);
        drop(behaviour);

        assert_eq!(key.public().to_peer_id(), peer_id);
    }
}
