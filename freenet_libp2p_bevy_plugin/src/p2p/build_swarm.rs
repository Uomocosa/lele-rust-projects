use std::time::Duration;

use libp2p::identity::Keypair;
use libp2p::{noise, tcp, yamux};

use crate::p2p;

pub fn build_swarm<T: p2p::Message>(
    keypair: Keypair,
) -> Result<libp2p::Swarm<p2p::Behaviour<T>>, p2p::Error> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|e| p2p::Error::Build(e.to_string()))?
        .with_quic()
        .with_dns()
        .map_err(|e| p2p::Error::Build(e.to_string()))?
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .map_err(|e| p2p::Error::Build(e.to_string()))?
        .with_behaviour(|key, relay| p2p::behaviour_new::new::<T>(key, relay))
        .map_err(|e| p2p::Error::Build(e.to_string()))?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(
                p2p::constants::IDLE_CONNECTION_TIMEOUT_SECS,
            ))
        })
        .build();
    Ok(swarm)
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};

    use libp2p::identity::Keypair;

    use super::build_swarm;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        assert!(build_swarm::<Dummy>(Keypair::generate_ed25519()).is_ok());
    }
}
