use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::roster;

use crate::boxes;

pub fn spawn_on_join(
    mut joins: MessageReader<roster::PeerJoined>,
    mut commands: Commands,
    identity: Res<freenet_libp2p_bevy_plugin::net_id::LocalIdentity>,
) {
    for join in joins.read() {
        let id = **join;
        if id == **identity {
            continue;
        }
        tracing::debug!(target: "roster", player = format!("{:08x}", *id as u32), "spawning remote box on join");
        boxes::spawn_box(
            &mut commands,
            boxes::Player(id),
            Vec2::new(boxes::spawn_x_for_player(id), boxes::SPAWN_Y),
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_on_join;
    use crate::boxes;
    use freenet_libp2p_bevy_plugin::{net_id, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<roster::PeerJoined>();
        app.insert_resource(net_id::LocalIdentity(net_id::NetworkId(99)));
        app.world_mut()
            .resource_mut::<bevy::ecs::message::Messages<roster::PeerJoined>>()
            .write(roster::PeerJoined(net_id::NetworkId(7)));
        app.add_systems(Update, spawn_on_join);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&boxes::Player, Option<&boxes::LocalPlayer>)>();
        let players: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(players.len(), 1);
        assert_eq!(**players[0].0, net_id::NetworkId(7));
        assert!(players[0].1.is_none());
    }
}
