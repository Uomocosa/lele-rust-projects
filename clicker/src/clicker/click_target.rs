use bevy::prelude::Component;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct ClickTarget;

#[cfg(test)]
mod tests {
    use super::ClickTarget;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut world = World::new();
        let entity = world.spawn(ClickTarget).id();
        assert!(world.get::<ClickTarget>(entity).is_some());
    }
}
