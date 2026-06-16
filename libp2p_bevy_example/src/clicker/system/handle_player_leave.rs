use bevy::prelude::*;

use crate::clicker::component;
use crate::p2p::NetworkEvent;

pub fn handle_player_leave(
    mut events: MessageReader<NetworkEvent>,
    mut commands: Commands,
    owner_query: Query<(Entity, &component::Owner)>,
) {
    for event in events.read() {
        if let NetworkEvent::PeerDisconnected(peer_id) = event {
            for (entity, owner) in owner_query.iter() {
                if owner.peer_id == *peer_id {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::clicker::component;
    use crate::clicker::system;
    use crate::p2p::NetworkEvent;
    use bevy::prelude::*;
    use libp2p::PeerId;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<NetworkEvent>();
        app.add_systems(Update, system::handle_player_leave);

        let peer_id = PeerId::random();
        app.world_mut().spawn(component::Owner { peer_id });

        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<NetworkEvent>>()
            .write(NetworkEvent::PeerDisconnected(peer_id));
        app.update();

        assert!(
            app.world_mut()
                .query::<&component::Owner>()
                .iter(app.world())
                .next()
                .is_none(),
            "Player should be despawned after leave event"
        );
    }
}
