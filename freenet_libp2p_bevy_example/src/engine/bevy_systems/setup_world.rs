use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::engine;

pub fn setup_world(mut commands: Commands) {
    commands.spawn((
        RigidBody::Static,
        Collider::rectangle(engine::GROUND_WIDTH, engine::GROUND_THICKNESS),
        Transform::from_xyz(0.0, engine::GROUND_Y - engine::GROUND_THICKNESS / 2.0, 0.0),
    ));

    let wall_center_y = engine::GROUND_Y + engine::WALL_HEIGHT / 2.0;
    for wall_x in [-engine::GROUND_WIDTH / 2.0, engine::GROUND_WIDTH / 2.0] {
        commands.spawn((
            RigidBody::Static,
            Collider::rectangle(engine::WALL_THICKNESS, engine::WALL_HEIGHT),
            Transform::from_xyz(wall_x, wall_center_y, 0.0),
        ));
    }
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::RigidBody;
    use bevy::prelude::*;

    use super::setup_world;
    use crate::engine;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_plugins(avian2d::prelude::PhysicsPlugins::default());
        app.add_systems(Startup, setup_world);
        app.update();

        let count = app
            .world_mut()
            .query::<&RigidBody>()
            .iter(app.world())
            .count();
        assert!(count >= 3);
        let _ = engine::WALL_HEIGHT;
    }
}
