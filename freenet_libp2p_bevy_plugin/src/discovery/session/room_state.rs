use bevy::prelude::States;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RoomState {
    #[default]
    Browsing,
    Joining,
    InRoom,
}

#[cfg(test)]
mod tests {
    use super::RoomState;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        assert_eq!(RoomState::default(), RoomState::Browsing);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<RoomState>();
        app.update();
        let state = app.world().resource::<State<RoomState>>();
        assert_eq!(**state, RoomState::Browsing);
    }
}
