use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

#[derive(Component, Debug, Default, Clone, Copy)]
struct ClickTarget;

pub fn spawn_target(
    commands: &mut Commands,
    owner: net_id::NetworkId,
    index: i32,
    is_local: bool,
) -> Entity {
    let color = if is_local {
        Color::srgb(0.2, 0.7, 0.3)
    } else {
        Color::hsl(
            f32::from(u16::try_from(*owner % 360).unwrap_or(0)),
            0.7,
            0.5,
        )
    };
    let x = f32::from(i16::try_from(index).unwrap_or(0)) * clicker::TARGET_SPACING;
    commands
        .spawn((
            clicker::Owner(owner),
            clicker::ClickCounter::default(),
            ClickTarget,
            Sprite::from_color(color, Vec2::splat(clicker::TARGET_SIZE)),
            Transform::from_translation(Vec3::new(x, clicker::ROW_Y, 0.0)),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::CommandQueue;
    use bevy::prelude::*;

    use super::ClickTarget;
    use super::spawn_target;
    use crate::clicker;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);

        let entity = spawn_target(&mut commands, net_id::NetworkId(7), 0, true);
        queue.apply(&mut world);

        assert!(world.get::<clicker::Owner>(entity).is_some());
        assert_eq!(
            **world.get::<clicker::Owner>(entity).unwrap(),
            net_id::NetworkId(7)
        );
        assert!(world.get::<ClickTarget>(entity).is_some());
    }
}
