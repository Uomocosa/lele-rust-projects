use bevy::prelude::*;

use crate::lobby;

pub fn build(_plugin: &lobby::Plugin, app: &mut App) {
    app.init_state::<lobby::AppState>()
        .init_resource::<lobby::RoomList>()
        .init_resource::<lobby::SelectedRoom>()
        .init_resource::<lobby::LeftRoom>()
        .init_resource::<lobby::JoinPending>()
        .init_resource::<lobby::JoinGate>()
        .init_resource::<lobby::JoinClock>()
        .init_resource::<lobby::DirectoryLive>()
        .add_systems(
            Update,
            (
                lobby::bevy_systems::poll_directory,
                lobby::bevy_systems::apply_room,
                lobby::bevy_systems::show_menu,
                lobby::bevy_systems::create_button,
                lobby::bevy_systems::join_button,
                lobby::bevy_systems::join_feedback,
            )
                .run_if(in_state(lobby::AppState::Menu)),
        )
        .add_systems(
            Update,
            (
                lobby::bevy_systems::poll_expected,
                lobby::bevy_systems::clear_pending,
            )
                .run_if(in_state(lobby::AppState::InRoom)),
        )
        .add_systems(
            Update,
            lobby::bevy_systems::leave_button.run_if(in_state(lobby::AppState::InRoom)),
        )
        .add_systems(
            OnEnter(lobby::AppState::InRoom),
            lobby::bevy_systems::spawn_leave,
        )
        .add_systems(
            OnExit(lobby::AppState::Menu),
            lobby::bevy_systems::despawn_menu,
        )
        .add_systems(
            OnEnter(lobby::AppState::Menu),
            lobby::bevy_systems::despawn_leave,
        );
}

#[cfg(test)]
mod tests {
    use super::build;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{net_id, p2p, roster};

    use crate::clicker;
    use crate::lobby;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_resource::<Time>();
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.insert_resource(p2p::Events::<clicker::CursorMsg>::default());
        app.insert_resource(roster::Roster::default());
        app.insert_resource(roster::Lobby::default());
        app.insert_resource(net_id::NetworkId(1));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(clicker::ActiveLobby::default());
        let (_dir_tx, dir_rx) = tokio::sync::mpsc::unbounded_channel();
        let (req_tx, _req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        let (_room_tx, room_rx) = tokio::sync::watch::channel(None::<String>);
        app.insert_resource(lobby::DirectoryFeed(std::sync::Mutex::new(Some(dir_rx))));
        app.insert_resource(lobby::RoomRequestTx(req_tx));
        app.insert_resource(lobby::RoomRx(std::sync::Mutex::new(Some(room_rx))));
        build(&lobby::Plugin, &mut app);
        app.update();
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::Menu);
    }
}
