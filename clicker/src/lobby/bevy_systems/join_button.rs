use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::lobby;

pub fn join_button(
    mut commands: Commands,
    triggers: Query<(&Interaction, &lobby::bevy_systems::RoomButton), Changed<Interaction>>,
    requests: Option<Res<lobby::RoomRequestTx>>,
    mut rooms: lobby::bevy_systems::JoinCtx,
    outbox: ResMut<p2p::Commands<clicker::CursorMsg>>,
    next: ResMut<NextState<lobby::AppState>>,
) {
    if rooms.pending.is_some() {
        return;
    }
    if !**rooms.live {
        return;
    }
    let mut room: Option<String> = None;
    for (interaction, button) in triggers.iter() {
        if *interaction == Interaction::Pressed {
            room = Some((**button).clone());
        }
    }
    let Some(room) = room else {
        return;
    };
    if lobby::create_room(&room).is_none() {
        return;
    }
    let outbox = outbox.into_inner();
    let next = next.into_inner();
    if let Some(requests) = requests {
        let requests = requests.into_inner();
        requests.send(room.clone()).ok();
    }
    lobby::join_room(&room, &mut rooms.rooms(), outbox);
    commands.spawn((
        lobby::bevy_systems::LoadingRoot,
        Text::new(format!("Joining {room}...")),
    ));
    next.set(lobby::AppState::InRoom);
}

#[cfg(test)]
mod tests {
    use super::join_button;
    use crate::clicker;
    use crate::lobby;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{p2p, roster};

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(lobby::SelectedRoom::default());
        app.insert_resource(roster::Lobby::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::JoinPending::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive(true));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        let (req_tx, mut req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        app.insert_resource(lobby::RoomRequestTx(req_tx));
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::RoomButton("room-a".to_string()),
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, join_button);
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(active, &clicker::ActiveLobby("room-a".to_string()));
        assert_eq!(req_rx.try_recv().unwrap_or_default(), "room-a");
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::InRoom);
        let outbox = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(outbox.len(), 5);
        let lobby = app.world().resource::<roster::Lobby>();
        assert_eq!(lobby.as_str(), "room-a");
        let pending = app.world().resource::<lobby::JoinPending>();
        assert_eq!(**pending, Some("room-a".to_string()));
        let overlays = app
            .world_mut()
            .query::<&lobby::bevy_systems::LoadingRoot>()
            .iter(app.world())
            .count();
        assert_eq!(overlays, 1, "click raises the loading overlay");
    }

    #[test]
    fn second_click_ignored_while_pending() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(lobby::SelectedRoom::default());
        app.insert_resource(roster::Lobby::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::JoinPending(Some("room-a".to_string())));
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive(true));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::RoomButton("room-b".to_string()),
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, join_button);
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(active, &clicker::ActiveLobby::default());
        let outbox = app.world().resource::<p2p::Commands<clicker::CursorMsg>>();
        assert_eq!(outbox.len(), 0);
        let overlays = app
            .world_mut()
            .query::<&lobby::bevy_systems::LoadingRoot>()
            .iter(app.world())
            .count();
        assert_eq!(overlays, 0, "ignored click raises no overlay");
    }

    #[test]
    fn click_ignored_while_directory_dark() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(lobby::SelectedRoom::default());
        app.insert_resource(roster::Lobby::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::JoinPending::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        let (req_tx, mut req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        app.insert_resource(lobby::RoomRequestTx(req_tx));
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::RoomButton("room-a".to_string()),
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, join_button);
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(
            active,
            &clicker::ActiveLobby::default(),
            "menu answers no clicks before the directory is live"
        );
        assert!(
            req_rx.try_recv().is_err(),
            "dark menu sends no room request"
        );
    }
}
