use bevy::prelude::Component;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MenuRoot;

#[cfg(test)]
mod tests {
    use super::MenuRoot;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(MenuRoot).id();
        app.update();
        assert!(app.world().get::<MenuRoot>(entity).is_some());
    }
}
