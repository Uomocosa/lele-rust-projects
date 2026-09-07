use bevy::prelude::Component;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct OwnScore;

#[cfg(test)]
mod tests {
    use super::OwnScore;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(OwnScore).id();
        assert!(world.get::<OwnScore>(entity).is_some());
    }
}
