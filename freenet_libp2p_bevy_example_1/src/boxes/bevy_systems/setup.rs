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

    boxes::spawn_box(
        &mut commands,
        boxes::Player(**config),
        Vec2::new(boxes::pick_spawn_x(&[]), boxes::SPAWN_Y),
        true,
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::setup;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(boxes::Config::new(boxes::PlayerId(9)));
        app.add_systems(Update, setup);
        app.update();

        let mut query = app
            .world_mut()
            .query::<(&boxes::LocalPlayer, &boxes::Player)>();
        let pairs: Vec<_> = query.iter(app.world()).collect();
        assert_eq!(pairs.len(), 1);
        assert_eq!(**pairs[0].1, boxes::PlayerId(9));
    }
}
