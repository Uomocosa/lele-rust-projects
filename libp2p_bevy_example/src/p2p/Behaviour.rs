use libp2p::gossipsub;
use libp2p::mdns::tokio::Behaviour as Mdns;
use libp2p::swarm::{behaviour::toggle::Toggle, NetworkBehaviour};

#[derive(NetworkBehaviour)]
#[behaviour(event_process = false)]
pub struct Behaviour {
    pub mdns: Toggle<Mdns>,
    pub gossipsub: gossipsub::Behaviour,
}
