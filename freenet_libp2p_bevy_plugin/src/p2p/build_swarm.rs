use libp2p::identity::Keypair;
use libp2p::kad::store::MemoryStore;
use libp2p::{StreamProtocol, identify, kad, noise, ping, request_response, tcp, yamux};

use crate::p2p;

/// # Errors
/// Returns error if swarm construction fails or the internal
/// replication factor is invalid.
pub fn build_swarm<T: p2p::Message>(
    keypair: Keypair,
) -> Result<libp2p::Swarm<p2p::Behaviour<T>>, String> {
    let peer_id = keypair.public().to_peer_id();
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| e.to_string())?
        .with_quic()
        .with_dns()
        .map_err(|e| e.to_string())?
        .with_behaviour(
            |kp| -> Result<p2p::Behaviour<T>, Box<dyn std::error::Error + Send + Sync>> {
                let mut kad_cfg = kad::Config::default();
                let Some(replication) = std::num::NonZeroUsize::new(8) else {
                    return Err("invalid replication factor".into());
                };
                kad_cfg.set_replication_factor(replication);
                let store = MemoryStore::new(kp.public().to_peer_id());
                let mut kademlia = kad::Behaviour::with_config(peer_id, store, kad_cfg);
                kademlia.set_mode(Some(kad::Mode::Server));
                let rr = request_response::Behaviour::with_codec(
                    p2p::MessageCodec::<T>::default(),
                    [(
                        StreamProtocol::new("/blackboard/1.0.0"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                );
                Ok(p2p::Behaviour {
                    request_response: rr,
                    kademlia,
                    identify: identify::Behaviour::new(identify::Config::new(
                        "/blackboard/id/1.0.0".to_string(),
                        kp.public(),
                    )),
                    ping: ping::Behaviour::default(),
                })
            },
        )
        .map_err(|e| e.to_string())?
        .build();
    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use libp2p::identity::Keypair;

    use super::build_swarm;
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let kp = Keypair::generate_ed25519();
        let swarm = build_swarm::<Dummy>(kp);
        assert!(swarm.is_ok());
    }
}
