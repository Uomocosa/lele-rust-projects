use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::lobby;

pub fn create_button(
    triggers: Query<
        &Interaction,
        (
            Changed<Interaction>,
            With<lobby::bevy_systems::CreateMarker>,
        ),
    >,
    requests: Option<Res<lobby::RoomRequestTx>>,
    active: ResMut<clicker::ActiveLobby>,
    selected: ResMut<lobby::SelectedRoom>,
    roster_lobby: ResMut<roster::Lobby>,
    outbox: ResMut<p2p::Commands<clicker::CursorMsg>>,
    next: ResMut<NextState<lobby::AppState>>,
) {
    let pressed = triggers
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed {
        return;
    }
    let active = active.into_inner();
    let selected = selected.into_inner();
    let roster_lobby = roster_lobby.into_inner();
    let outbox = outbox.into_inner();
    let next = next.into_inner();
    let Some(room) = lobby::create_room(&format!("room-{}", epoch_secs())) else {
        return;
    };
    if let Some(requests) = requests {
        let requests = requests.into_inner();
        requests.send(room.clone()).ok();
    }
    lobby::join_room(&room, active, selected, roster_lobby, outbox);
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
}
