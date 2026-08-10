use avian2d::prelude::LinearVelocity;

use crate::boxes;

pub fn jump_box(velocity: &mut LinearVelocity, grounded: bool) -> bool {
    if !grounded {
        return false;
    }
    velocity.0.y = boxes::JUMP_SPEED;
    true
}

#[cfg(test)]
mod tests {
    use avian2d::prelude::LinearVelocity;

    use super::jump_box;

    #[test]
    fn test_usage() {
        let mut velocity = LinearVelocity::ZERO;

        assert!(!jump_box(&mut velocity, false));
        assert_eq!(velocity.0.y, 0.0);

        assert!(jump_box(&mut velocity, true));
        assert!(velocity.0.y > 0.0);
    }
}
