use bevy::prelude::States;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Menu,
    InRoom,
}

#[cfg(test)]
mod tests {
    use super::AppState;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        assert_eq!(AppState::default(), AppState::Menu);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<AppState>();
        app.update();
        let state = app.world().resource::<State<AppState>>();
        assert_eq!(**state, AppState::Menu);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InRoom);
        app.update();
        let state = app.world().resource::<State<AppState>>();
        assert_eq!(**state, AppState::InRoom);
    }
}
