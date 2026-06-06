use futures::StreamExt;
use libp2p::{
    core::Transport, identity, noise, ping, swarm::SwarmEvent, tcp, yamux, Multiaddr,
};
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let keypair = identity::Keypair::generate_ed25519();
    let peer_id = keypair.public().to_peer_id();
    info!(%peer_id, "libp2p peer ID");

    let transport = tcp::tokio::Transport::default()
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(&keypair)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let behaviour = ping::Behaviour::new(ping::Config::new());
    let config = libp2p::swarm::Config::with_tokio_executor();
    let mut swarm = libp2p::Swarm::new(transport, behaviour, peer_id, config);

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse::<Multiaddr>()?)?;

    // TODO: Also start a freenet node alongside libp2p to prove coexistence.
    // See freenet's local_node module for the node startup API.

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                info!(%address, "Listening on");
            }
            SwarmEvent::Behaviour(ping::Event { peer, result, .. }) => {
                match result {
                    Ok(rtt) => info!(%peer, ?rtt, "Ping OK"),
                    Err(e) => info!(%peer, error = %e, "Ping failed"),
                }
            }
            _ => {}
        }
    }
}
