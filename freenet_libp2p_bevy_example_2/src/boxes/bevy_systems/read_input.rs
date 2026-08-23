use avian2d::prelude::{LinearVelocity, Position, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;

use crate::boxes;

pub fn read_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    spatial_query: SpatialQuery,
    mut query: Query<(Entity, &Position, &mut LinearVelocity), With<boxes::LocalPlayer>>,
) {
    let left = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);
    let direction = match (left, right) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };
    let jump_pressed = keyboard.just_pressed(KeyCode::Space);

    for (entity, position, mut velocity) in &mut query {
        boxes::move_box(&mut velocity, direction);

        if jump_pressed {
            let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
            let grounded = spatial_query
                .cast_ray(
                    position.0,
                    Dir2::NEG_Y,
                    boxes::GROUND_CHECK_DISTANCE,
                    true,
                    &filter,
                )
                .is_some();
            boxes::jump_box(&mut velocity, grounded);
        }
    }
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::*;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::read_input;
    use crate::boxes;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_plugins(PhysicsPlugins::default());
        app.insert_resource(ButtonInput::<KeyCode>::default());

        let entity = app
            .world_mut()
            .spawn((
                boxes::LocalPlayer,
                boxes::Player([0; 32]),
                RigidBody::Dynamic,
                Collider::rectangle(boxes::BOX_SIZE, boxes::BOX_SIZE),
                LinearVelocity::ZERO,
                Transform::default(),
            ))
            .id();

        app.add_systems(Update, read_input);
        app.update();

        let velocity = app.world().get::<LinearVelocity>(entity).unwrap();
        assert_eq!(velocity.0.x, 0.0);
    }

    #[test]
    fn test_moves_on_key_press() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_plugins(PhysicsPlugins::default());
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(1.0 / 60.0),
        ));
        app.finish();
        app.cleanup();

        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyD);
        app.insert_resource(input);

        let entity = app
            .world_mut()
            .spawn((
                boxes::LocalPlayer,
                boxes::Player([0; 32]),
                RigidBody::Dynamic,
                Collider::rectangle(boxes::BOX_SIZE, boxes::BOX_SIZE),
                LinearVelocity::ZERO,
                Position::new(Vec2::ZERO),
                Transform::default(),
            ))
            .id();

        app.add_systems(Update, read_input);

        for _ in 0..20 {
            app.update();
        }

        let position = app.world().get::<Position>(entity).unwrap();
        assert!(position.x > 0.0);
        assert_ne!((position.x, position.y), (0.0, 0.0));
    }
}
