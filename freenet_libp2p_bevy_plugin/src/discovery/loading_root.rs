use bevy::prelude::Component;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LoadingRoot;

#[cfg(test)]
mod tests {
    use super::LoadingRoot;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(LoadingRoot).id();
        app.update();
        assert!(app.world().get::<LoadingRoot>(entity).is_some());
    }
}
