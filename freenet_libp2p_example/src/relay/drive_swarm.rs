use futures::StreamExt;
use libp2p::Swarm;
use libp2p::swarm::SwarmEvent;

use crate::relay::{Behaviour, GossipState};

pub async fn drive_swarm(swarm: &mut Swarm<Behaviour>, _gossip: &mut GossipState) {
    let _ = swarm.select_next_some().await;
    if let SwarmEvent::Behaviour(_) = swarm.select_next_some().await {}
}

// no test_usage necessary
