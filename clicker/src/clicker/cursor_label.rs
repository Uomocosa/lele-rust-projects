use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct CursorLabel;

#[cfg(test)]
mod tests {
    use super::CursorLabel;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(CursorLabel).id();
        assert!(world.get::<CursorLabel>(entity).is_some());
    }
}
