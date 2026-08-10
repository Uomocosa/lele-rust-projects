use avian2d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody};
use bevy::prelude::*;

use crate::boxes;

pub fn spawn_box(
    commands: &mut Commands,
    player: boxes::Player,
    position: Vec2,
    is_local: bool,
) -> Entity {
    let hue = (**player % 360) as f32;
    let color = Color::hsl(hue, 0.7, 0.5);

    let mut entity = commands.spawn((
        Sprite::from_color(color, Vec2::splat(boxes::BOX_SIZE)),
        Transform::from_translation(position.extend(0.0)),
        RigidBody::Dynamic,
        Collider::rectangle(boxes::BOX_SIZE, boxes::BOX_SIZE),
        LockedAxes::ROTATION_LOCKED,
        LinearVelocity::ZERO,
        player,
    ));

    if is_local {
        entity.insert(boxes::LocalPlayer);
    }

    entity.id()
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;

    use super::spawn_box;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let entity = spawn_box(
            &mut commands,
            boxes::Player(boxes::PlayerId(1)),
            Vec2::ZERO,
            true,
        );
        queue.apply(&mut world);

        assert!(world.get::<boxes::LocalPlayer>(entity).is_some());
        assert!(world.get::<boxes::Player>(entity).is_some());
    }
}
