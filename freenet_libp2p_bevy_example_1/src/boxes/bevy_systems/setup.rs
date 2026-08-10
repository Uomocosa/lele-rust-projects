use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::boxes;

pub fn setup(mut commands: Commands) {
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
        boxes::Player {
            id: boxes::PlayerId { value: 0 },
        },
        Vec2::new(0.0, boxes::GROUND_Y + boxes::BOX_SIZE),
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
        app.add_systems(Update, setup);
        app.update();

        let mut query = app.world_mut().query::<&boxes::LocalPlayer>();
        assert_eq!(query.iter(app.world()).count(), 1);
    }
}
