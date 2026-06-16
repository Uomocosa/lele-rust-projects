use tracing::debug;

use crate::sync::resource;

pub fn log_peer_count(network_state: bevy::prelude::Res<resource::NetworkState>) {
    let count = network_state.connected_peers.len();
    if count > 0 {
        debug!("Connected peers: {}", count);
    }
}

#[cfg(test)]
mod tests {
    use super::log_peer_count;
    use crate::sync::resource;
    use bevy::ecs::schedule::Schedule;
    use bevy::prelude::World;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        world.init_resource::<resource::NetworkState>();
        let mut schedule = Schedule::default();
        schedule.add_systems(log_peer_count);
        schedule.run(&mut world);
    }

    #[test]
    fn test_logs_when_peers_connected() {
        let mut world = World::new();
        let mut network_state = resource::NetworkState::default();
        network_state.connected_peers.push(PeerId::random());
        world.insert_resource(network_state);
        let mut schedule = Schedule::default();
        schedule.add_systems(log_peer_count);
        schedule.run(&mut world);
    }
}
