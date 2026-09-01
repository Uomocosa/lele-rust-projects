use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;

use crate::engine;

pub fn apply_pending_actions(
    pending: Res<engine::PendingActions>,
    mut query: Query<(&engine::Player, &Position, &mut LinearVelocity)>,
) {
    for (player, position, mut velocity) in &mut query {
        let Some(action) = pending.0.get(&player.0) else {
            continue;
        };
        let vx = action.move_value() * engine::MOVE_SPEED;
        let mut vy = velocity.y;

        if action.jump && engine::is_grounded(position.0.y) {
            vy = engine::JUMP_SPEED;
        }

        velocity.x = vx;
        velocity.y = vy;
    }
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::*;
    use bevy::prelude::*;

    use super::apply_pending_actions;
    use crate::engine;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::transform::TransformPlugin);
        app.add_plugins(PhysicsPlugins::default());
        app.insert_resource(engine::PendingActions::default());
        app.add_systems(Update, apply_pending_actions);

        app.world_mut().spawn((
            engine::Player([0; 32]),
            RigidBody::Dynamic,
            Collider::rectangle(engine::BOX_SIZE, engine::BOX_SIZE),
            LinearVelocity::ZERO,
            Transform::default(),
        ));
        app.update();

        let players = app
            .world_mut()
            .query::<&engine::Player>()
            .iter(app.world())
            .count();
        assert_eq!(players, 1);
    }
}
