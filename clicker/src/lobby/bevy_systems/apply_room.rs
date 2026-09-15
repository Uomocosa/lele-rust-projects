use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::p2p;

use crate::clicker;
use crate::lobby;

pub fn apply_room(
    mut commands: Commands,
    roots: Query<Entity, With<lobby::bevy_systems::MenuRoot>>,
    feed: Res<lobby::RoomRx>,
    active: ResMut<clicker::ActiveLobby>,
    selected: ResMut<lobby::SelectedRoom>,
    outbox: ResMut<p2p::Commands<clicker::CursorMsg>>,
    next: ResMut<NextState<lobby::AppState>>,
) {
    let room = feed
        .into_inner()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|rx| rx.borrow().clone()));
    let Some(room) = room else {
        return;
    };
    if **active == room {
        return;
    }
    let active = active.into_inner();
    let selected = selected.into_inner();
    let outbox = outbox.into_inner();
    let next = next.into_inner();
    lobby::join_room(&room, active, selected, outbox);
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    next.set(lobby::AppState::InRoom);
}

#[cfg(test)]
mod tests {
    use super::apply_room;
    use crate::clicker;
    use crate::lobby;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::p2p;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::watch::channel(None::<String>);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.insert_resource(lobby::RoomRx(Mutex::new(Some(rx))));
        app.insert_resource(clicker::ActiveLobby::default());
        app.insert_resource(lobby::SelectedRoom::default());
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut()
            .spawn((lobby::bevy_systems::MenuRoot, Text::new("menu")));
        app.add_systems(Update, apply_room);
        app.update();
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::Menu);
        tx.send_replace(Some("room-auto".to_string()));
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(active, &clicker::ActiveLobby("room-auto".to_string()));
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::InRoom);
    }
}
