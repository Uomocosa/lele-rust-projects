use bevy::prelude::Component;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LeaveMarker;

#[cfg(test)]
mod tests {
    use super::LeaveMarker;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(LeaveMarker).id();
        app.update();
        assert!(app.world().get::<LeaveMarker>(entity).is_some());
    }
}
