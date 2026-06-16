use bevy::prelude::*;
use libp2p::PeerId;

use crate::boxes::component;
use crate::p2p::NetworkEvent;

pub fn handle_player_join(mut events: MessageReader<NetworkEvent>, mut commands: Commands) {
    for event in events.read() {
        if let NetworkEvent::PeerConnected(peer_id) = event {
            spawn_remote_player(&mut commands, *peer_id);
        }
    }
}

pub(crate) fn spawn_remote_player(commands: &mut Commands, peer_id: PeerId) {
    commands.spawn((
        component::Player {
            peer_id,
            is_local: false,
        },
        component::Position::zero(),
        component::Velocity::zero(),
        component::PlayerInput::new(),
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(32.0, 32.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 0.0),
    ));
}

#[cfg(test)]
mod tests {
    use crate::boxes::component;
    use crate::boxes::system;
    use bevy::prelude::World;
    use libp2p::PeerId;

    #[test]
    fn test_usage() -> Result<(), Box<dyn std::error::Error>> {
        let mut world = World::new();
        let peer_id = PeerId::random();

        {
            let mut commands = world.commands();
            system::handle_player_join::spawn_remote_player(&mut commands, peer_id);
        }
        world.flush();

        let mut query = world.query::<&component::Player>();
        let player = query.single(&world).map_err(|e| format!("{e:?}"))?;
        assert!(player.peer_id == peer_id);
        Ok(())
    }
}
