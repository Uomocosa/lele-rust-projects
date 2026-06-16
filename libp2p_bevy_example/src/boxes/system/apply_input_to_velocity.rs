use crate::boxes::component;
use crate::p2p::PlayerInputData;

const MOVE_SPEED: f32 = 200.0;
const JUMP_VELOCITY: f32 = 350.0;
const UP_SPEED: f32 = 150.0;
const GROUND_Y: f32 = -200.0;

pub fn apply_input_to_velocity(
    input: &PlayerInputData,
    velocity: &mut component::Velocity,
    position: &component::Position,
) {
    if input.left && !input.right {
        velocity.x = -MOVE_SPEED;
    } else if input.right && !input.left {
        velocity.x = MOVE_SPEED;
    } else {
        velocity.x = 0.0;
    }

    if input.up {
        velocity.y = UP_SPEED;
    } else if input.jump && position.y <= GROUND_Y + 1.0 {
        velocity.y = JUMP_VELOCITY;
    }
}

#[cfg(test)]
mod tests {
    use crate::boxes::component;
    use crate::boxes::system;
    use crate::p2p::PlayerInputData;

    #[test]

    fn test_usage() {
        let mut velocity = component::Velocity::zero();

        let position = component::Position::new(0.0, -200.0);

        let input = PlayerInputData::from_bools(true, false, false, false);

        system::apply_input_to_velocity(&input, &mut velocity, &position);

        assert_eq!(velocity.x, -200.0, "Should move left");
    }
}
