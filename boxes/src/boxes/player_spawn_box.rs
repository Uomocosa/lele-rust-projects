use avian2d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::prelude::*;

use super::player::Player;
use crate::boxes;

#[derive(Component, Debug, Default, Clone, Copy)]
struct LocalPlayer;

pub fn spawn_box(
    player: Player,
    commands: &mut Commands,
    position: Vec2,
    is_local: bool,
) -> Entity {
    let hue = f32::from(u16::try_from(**player % 360).unwrap_or(0));
    let color = Color::hsl(hue, 0.7, 0.5);
    let body = if is_local {
        RigidBody::Dynamic
    } else {
        RigidBody::Kinematic
    };

    let mut entity = commands.spawn((
        Sprite::from_color(color, Vec2::splat(boxes::BOX_SIZE)),
        Transform::from_translation(position.extend(0.0)),
        body,
        Collider::rectangle(boxes::BOX_SIZE, boxes::BOX_SIZE),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
        player,
    ));

    if is_local {
        entity.insert(LocalPlayer);
    }

    entity.id()
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::RigidBody;
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;

    use super::LocalPlayer;
    use crate::boxes;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let entity = super::super::player::Player(net_id::NetworkId(1)).spawn_box(
            &mut commands,
            Vec2::ZERO,
            true,
        );
        queue.apply(&mut world);

        assert!(world.get::<LocalPlayer>(entity).is_some());
        assert!(world.get::<boxes::Player>(entity).is_some());
        assert_eq!(world.get::<RigidBody>(entity), Some(&RigidBody::Dynamic));
    }

    #[test]
    fn remote_box_is_kinematic() {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let entity = super::super::player::Player(net_id::NetworkId(2)).spawn_box(
            &mut commands,
            Vec2::ZERO,
            false,
        );
        queue.apply(&mut world);

        assert!(world.get::<LocalPlayer>(entity).is_none());
        assert_eq!(world.get::<RigidBody>(entity), Some(&RigidBody::Kinematic));
    }
}
