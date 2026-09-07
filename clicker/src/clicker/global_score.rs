use bevy::prelude::Component;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct GlobalScore;

#[cfg(test)]
mod tests {
    use super::GlobalScore;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(GlobalScore).id();
        assert!(world.get::<GlobalScore>(entity).is_some());
    }
}
