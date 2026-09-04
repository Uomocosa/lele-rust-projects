use avian2d::prelude::LinearVelocity;

use crate::boxes;

pub fn move_box(velocity: &mut LinearVelocity, direction: f32) {
    velocity.0.x = direction.clamp(-1.0, 1.0) * boxes::MOVE_SPEED;
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::LinearVelocity;

    use super::move_box;

    #[test]
    fn test_usage() {
        let mut velocity = LinearVelocity::ZERO;

        move_box(&mut velocity, 1.0);
        assert!(velocity.0.x > 0.0);

        move_box(&mut velocity, -1.0);
        assert!(velocity.0.x < 0.0);

        move_box(&mut velocity, 0.0);
        assert_eq!(velocity.0.x, 0.0);
    }
}
