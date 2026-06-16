use bevy::prelude::*;
use tracing::{error, info};

use crate::p2p::resource::PeerState;
use crate::p2p::resource::Session;
use crate::p2p::system;
use crate::p2p::Plugin;
use crate::p2p::Swarm;

pub fn build(plugin: &Plugin, app: &mut App) {
    let config = plugin.config.clone();

    let (swarm, event_receiver) = match Swarm::new(config.clone()) {
        Ok((s, r)) => (s, r),
        Err(e) => {
            error!("Failed to create P2P swarm: {}", e);
            return;
        }
    };

    let local_peer_id = swarm.local_peer_id;

    info!("P2P Plugin initialized with peer ID: {}", local_peer_id);

    app.insert_resource(Session {
        swarm,
        event_receiver,
    })
    .insert_resource(PeerState::new(&config, local_peer_id))
    .add_systems(FixedUpdate, system::poll_network)
    .add_systems(FixedUpdate, system::log_peer_count);
}

#[cfg(test)]
mod tests {
    use crate::p2p::Config;
    use crate::p2p::Plugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let plugin = Plugin::new(Config::default());
        let mut app = App::new();
        app.add_plugins(plugin);
    }
}
