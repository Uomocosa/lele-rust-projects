use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::boxes;

pub fn setup(mut commands: Commands, config: Res<boxes::Config>) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(
            Color::srgb(0.3, 0.3, 0.3),
            Vec2::new(boxes::GROUND_WIDTH, boxes::GROUND_THICKNESS),
        ),
        Transform::from_xyz(0.0, boxes::GROUND_Y - boxes::GROUND_THICKNESS / 2.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(boxes::GROUND_WIDTH, boxes::GROUND_THICKNESS),
    ));

    let wall_center_y = boxes::GROUND_Y + boxes::WALL_HEIGHT / 2.0;
    for wall_x in [-boxes::GROUND_WIDTH / 2.0, boxes::GROUND_WIDTH / 2.0] {
        commands.spawn((
            Sprite::from_color(
                Color::srgb(0.4, 0.4, 0.4),
                Vec2::new(boxes::WALL_THICKNESS, boxes::WALL_HEIGHT),
            ),
            Transform::from_xyz(wall_x, wall_center_y, 0.0),
            RigidBody::Static,
            Collider::rectangle(boxes::WALL_THICKNESS, boxes::WALL_HEIGHT),
        ));
    }

    boxes::spawn_box(
        &mut commands,
        boxes::Player(**config),
        Vec2::new(boxes::spawn_x_for_player(**config), boxes::SPAWN_Y),
        true,
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::setup;
    use crate::boxes;
    use freenet_libp2p_bevy_plugin::net_id;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(boxes::Config::new(net_id::NetworkId(9)));
        app.add_systems(Update, setup);
        app.update();

        let mut query = app
            .world_mut()
            .query::<(&boxes::LocalPlayer, &boxes::Player, &Transform)>();
        let pairs: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(**pairs[0].1, net_id::NetworkId(9));
        assert_eq!(
            pairs[0].2.translation.x,
            boxes::spawn_x_for_player(net_id::NetworkId(9))
        );
    }
}
