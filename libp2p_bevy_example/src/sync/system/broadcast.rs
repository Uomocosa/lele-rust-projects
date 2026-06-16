use bevy::prelude::*;
use tracing::debug;

use crate::boxes::component::Player;
use crate::boxes::component::PlayerInput;
use crate::p2p;
use crate::p2p::resource::Session;
use crate::p2p::Message;
use crate::sync::resource::NetworkState;
use crate::sync::resource::Tick;

pub fn broadcast(
    mut swarm_state: ResMut<Session>,
    network: Res<NetworkState>,
    tick: Res<Tick>,
    local_player_query: Query<(&Player, &PlayerInput)>,
) {
    let current_tick = tick.current;

    for (player, input) in &local_player_query {
        if !player.is_local {
            continue;
        }

        if input.input.is_zero() {
            continue;
        }

        let topic = p2p::get_game_topic();
        let message = Message::PlayerInput {
            tick: current_tick,
            input: input.input.clone(),
        };

        swarm_state.swarm.publish(topic, message);

        debug!(
            "Broadcast from {} for tick {}: input={:?}",
            network.local_peer_id, current_tick, input.input
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::p2p::resource::Session;
    use crate::p2p::Config;
    use crate::p2p::Swarm;
    use crate::sync::resource::NetworkState;
    use crate::sync::resource::Tick;
    use crate::sync::system;
    use bevy::prelude::*;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let config = Config::default();
        let (swarm, event_receiver) = Swarm::new(config)?;
        let mut app = App::new();
        app.insert_resource(Session {
            swarm,
            event_receiver,
        })
        .insert_resource(NetworkState::default())
        .insert_resource(Tick::default())
        .add_systems(FixedUpdate, system::broadcast);
        app.update();
        Ok(())
    }
}
