use bevy::prelude::Component;

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct JoinSpinner;

#[cfg(test)]
mod tests {
    use super::JoinSpinner;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(JoinSpinner).id();
        app.update();
        let spinner = app.world().get::<JoinSpinner>(entity).expect("spinner");
        assert_eq!(*spinner, JoinSpinner);
    }
}
