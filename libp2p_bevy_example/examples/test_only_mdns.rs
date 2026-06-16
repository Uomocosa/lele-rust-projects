use futures::StreamExt;
use std::time::Duration;
use tracing::info;

use libp2p::identity;
use libp2p::mdns::{tokio::Behaviour as Mdns, Config};
use libp2p::noise;
use libp2p::swarm::{Config as SwarmConfig, NetworkBehaviour, Swarm};
use libp2p::tcp;
use libp2p::yamux;
use libp2p::PeerId;
use libp2p::Transport;

#[derive(NetworkBehaviour)]
#[behaviour(event_process = false)]
struct TestBehaviour {
    mdns: Mdns,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    info!("Starting mDNS test...");

    let mut swarm1 = build_swarm().await?;
    let mut swarm2 = build_swarm().await?;

    let peer1_id = *swarm1.local_peer_id();
    let peer2_id = *swarm2.local_peer_id();

    info!("Swarm1 peer ID: {}", peer1_id);
    info!("Swarm2 peer ID: {}", peer2_id);

    info!("Polling for mDNS discovery (10 seconds)...");

    let mut found_1_to_2 = false;
    let mut found_2_to_1 = false;

    for i in 0..100 {
        tokio::select! {
            event1 = swarm1.next() => {
                if let Some(ev) = event1 {
                    tracing::debug!("[{}] Swarm1 event: {:?}", i, ev);
                }
            }
            event2 = swarm2.next() => {
                if let Some(ev) = event2 {
                    tracing::debug!("[{}] Swarm2 event: {:?}", i, ev);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }

        let peers1: Vec<_> = swarm1.behaviour().mdns.discovered_nodes().collect();
        let peers2: Vec<_> = swarm2.behaviour().mdns.discovered_nodes().collect();

        if !peers1.is_empty() {
            tracing::debug!("Swarm1 discovered: {:?}", peers1);
            if peers1.iter().any(|p| **p == peer2_id) {
                found_1_to_2 = true;
            }
        }

        if !peers2.is_empty() {
            tracing::debug!("Swarm2 discovered: {:?}", peers2);
            if peers2.iter().any(|p| **p == peer1_id) {
                found_2_to_1 = true;
            }
        }

        if found_1_to_2 && found_2_to_1 {
            info!("SUCCESS: Bidirectional mDNS discovery achieved!");
            break;
        }
    }

    if found_1_to_2 && found_2_to_1 {
        info!("=== BIDIRECTIONAL mDNS DISCOVERY WORKS ===");
        Ok(())
    } else {
        tracing::error!("=== mDNS DID NOT WORK ===");
        tracing::error!("found_1_to_2: {}", found_1_to_2);
        tracing::error!("found_2_to_1: {}", found_2_to_1);
        Err("mDNS discovery failed".into())
    }
}

async fn build_swarm() -> Result<Swarm<TestBehaviour>, Box<dyn std::error::Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(&local_key.public());

    let mdns = Mdns::new(Config::default(), peer_id)?;

    let transport = tcp::tokio::Transport::new(tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let behaviour = TestBehaviour { mdns };

    let mut swarm = Swarm::new(
        transport,
        behaviour,
        peer_id,
        SwarmConfig::without_executor(),
    );
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?).ok();

    Ok(swarm)
}
