use bevy::prelude::*;

use crate::boxes::component;
use crate::p2p::NetworkEvent;

pub fn handle_player_leave(
    mut events: MessageReader<NetworkEvent>,
    mut commands: Commands,
    player_query: Query<(Entity, &component::Player)>,
) {
    for event in events.read() {
        if let NetworkEvent::PeerDisconnected(peer_id) = event {
            for (entity, player) in player_query.iter() {
                if player.peer_id == *peer_id {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component;
    use crate::boxes::system;
    use crate::p2p::NetworkEvent;
    use bevy::prelude::*;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<NetworkEvent>();
        app.add_systems(Update, system::handle_player_leave);

        let peer_id = PeerId::random();
        app.world_mut().spawn(component::Player {
            peer_id,
            is_local: false,
        });

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<NetworkEvent>>()
            .write(NetworkEvent::PeerDisconnected(peer_id));
        app.update();

        assert!(
            app.world_mut()
                .query::<&component::Player>()
                .iter(app.world())
                .next()
                .is_none(),
            "Player should be despawned after leave event"
        );
    }
}
