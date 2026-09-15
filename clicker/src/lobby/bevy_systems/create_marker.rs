use bevy::prelude::Component;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CreateMarker;

#[cfg(test)]
mod tests {
    use super::CreateMarker;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(CreateMarker).id();
        app.update();
        assert!(app.world().get::<CreateMarker>(entity).is_some());
    }
}
