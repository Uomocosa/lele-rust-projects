use bevy::prelude::*;
use tracing;

use crate::boxes::component;

const MOVE_SPEED: f32 = 200.0;
const JUMP_VELOCITY: f32 = 350.0;
const UP_SPEED: f32 = 150.0;
const GRAVITY: f32 = 800.0;
const GROUND_Y: f32 = -200.0;

pub fn character_controller(
    mut query: Query<
        (
            &mut component::Position,
            &mut component::Velocity,
            &component::PlayerInput,
        ),
        With<component::Player>,
    >,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (mut pos, mut vel, input) in &mut query {
        if input.input.left && !input.input.right {
            vel.x = -MOVE_SPEED;
        } else if input.input.right && !input.input.left {
            vel.x = MOVE_SPEED;
        } else {
            vel.x = 0.0;
        }

        if input.input.up {
            vel.y = UP_SPEED;
        } else if input.input.jump && pos.y <= GROUND_Y + 1.0 {
            vel.y = JUMP_VELOCITY;
        } else if pos.y > GROUND_Y {
            vel.y -= GRAVITY * dt;
        } else {
            vel.y = 0.0;
            pos.y = GROUND_Y;
        }

        pos.x += vel.x * dt;
        pos.y += vel.y * dt;

        if pos.y < GROUND_Y {
            pos.y = GROUND_Y;
            vel.y = 0.0;
        }

        tracing::trace!(target: "character_controller", vel_x = vel.x, vel_y = vel.y, pos_x = pos.x, pos_y = pos.y);
    }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component;
    use crate::boxes::system;
    use bevy::ecs::schedule::Schedule;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();

        world.spawn((
            component::Position::new(0.0, -200.0),
            component::Velocity::zero(),
            component::PlayerInput::new(),
        ));

        world.insert_resource(Time::<Fixed>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems(system::character_controller);
        schedule.run(&mut world);

        let mut query = world.query::<&component::Position>();
        let positions: Vec<_> = query.iter(&world).collect();

        assert!(!positions.is_empty(), "Player should exist after update");
    }
}
