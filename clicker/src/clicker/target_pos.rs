use bevy::prelude::*;
use derive_more::{Deref, DerefMut};

#[derive(Component, Debug, Clone, Copy, Deref, DerefMut)]
pub struct TargetPos(pub Vec2);

#[cfg(test)]
mod tests {
    use super::TargetPos;
    use bevy::prelude::Vec2;

    #[test]
    fn test_usage() {
        let target = TargetPos(Vec2::new(2.5, 4.5));
        assert!(target.x > 2.25);
        assert!(target.x < 2.75);
        assert!(target.y > 4.25);
        assert!(target.y < 4.75);
    }
}
