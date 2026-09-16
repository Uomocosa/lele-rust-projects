use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::lobby;

pub fn join_button(
    triggers: Query<(&Interaction, &lobby::bevy_systems::RoomButton), Changed<Interaction>>,
    requests: Option<Res<lobby::RoomRequestTx>>,
    active: ResMut<clicker::ActiveLobby>,
    selected: ResMut<lobby::SelectedRoom>,
    roster_lobby: ResMut<roster::Lobby>,
    outbox: ResMut<p2p::Commands<clicker::CursorMsg>>,
    next: ResMut<NextState<lobby::AppState>>,
) {
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
    let active = active.into_inner();
    let selected = selected.into_inner();
    let roster_lobby = roster_lobby.into_inner();
    let outbox = outbox.into_inner();
    let next = next.into_inner();
    if let Some(requests) = requests {
        let requests = requests.into_inner();
        requests.send(room.clone()).ok();
    }
    lobby::join_room(&room, active, selected, roster_lobby, outbox);
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
        assert_eq!(outbox.len(), 4);
        let lobby = app.world().resource::<roster::Lobby>();
        assert_eq!(lobby.as_str(), "room-a");
    }
}
