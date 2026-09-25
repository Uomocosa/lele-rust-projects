use bevy::prelude::Component;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LeaveRoot;

#[cfg(test)]
mod tests {
    use super::LeaveRoot;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(LeaveRoot).id();
        app.update();
        assert!(app.world().get::<LeaveRoot>(entity).is_some());
    }
}
