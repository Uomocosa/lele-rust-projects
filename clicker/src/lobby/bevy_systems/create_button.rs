use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::lobby;

pub fn create_button(
    mut commands: Commands,
    triggers: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<lobby::bevy_systems::CreateMarker>,
        ),
    >,
    requests: Option<Res<lobby::RoomRequestTx>>,
    mut rooms: lobby::bevy_systems::JoinCtx,
    outbox: ResMut<p2p::Commands<clicker::CursorMsg>>,
    next: ResMut<NextState<lobby::AppState>>,
) {
    let pressed = triggers
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed {
        return;
    }
    if !**rooms.live {
        return;
    }
    let outbox = outbox.into_inner();
    let next = next.into_inner();
    let Some(room) = lobby::create_room(&format!("room-{}", epoch_secs())) else {
        return;
    };
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

// needed helper: seconds since the unix epoch for generated room names
fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::create_button;
    use crate::clicker;
    use crate::lobby;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(lobby::SelectedRoom::default());
        app.insert_resource(freenet_libp2p_bevy_plugin::roster::Lobby::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::JoinPending::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive(true));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        let (req_tx, mut req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        app.insert_resource(lobby::RoomRequestTx(req_tx));
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::CreateMarker,
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, create_button);
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert!(active.starts_with("room-"));
        let request = req_rx.try_recv().unwrap_or_default();
        assert_eq!(request, **active);
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::InRoom);
    }

    #[test]
    fn create_ignored_while_directory_dark() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(lobby::SelectedRoom::default());
        app.insert_resource(freenet_libp2p_bevy_plugin::roster::Lobby::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::JoinPending::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        let (req_tx, mut req_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        app.insert_resource(lobby::RoomRequestTx(req_tx));
        app.world_mut().spawn((
            lobby::bevy_systems::MenuRoot,
            lobby::bevy_systems::CreateMarker,
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, create_button);
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(
            active,
            &clicker::ActiveLobby::default(),
            "dark menu creates no room"
        );
        assert!(
            req_rx.try_recv().is_err(),
            "dark menu sends no room request"
        );
    }
}
