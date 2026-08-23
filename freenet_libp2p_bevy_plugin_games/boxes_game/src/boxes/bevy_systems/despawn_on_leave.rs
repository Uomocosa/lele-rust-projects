use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::roster;

use crate::boxes;

pub fn despawn_on_leave(
    mut leaves: MessageReader<roster::PeerLeft>,
    mut commands: Commands,
    remote: Query<(Entity, &boxes::Player), Without<boxes::LocalPlayer>>,
) {
    for leave in leaves.read() {
        let id = **leave;
        for (entity, player) in &remote {
            if **player == id {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::despawn_on_leave;
    use crate::boxes;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<roster::PeerLeft>();
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<roster::PeerLeft>>()
            .write(roster::PeerLeft(net_id::NetworkId(7)));
        let entity = app
            .world_mut()
            .spawn(boxes::Player(net_id::NetworkId(7)))
            .id();
        app.add_systems(Update, despawn_on_leave);

        app.update();

        assert!(app.world().get_entity(entity).is_err());
    }
}
