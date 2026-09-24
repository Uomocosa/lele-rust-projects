#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use crate::roster;

use super::super::active_room::ActiveRoom;
use super::super::join_clock::JoinClock;
use super::super::join_gate::JoinGate;
use super::super::join_pending::JoinPending;
use super::super::left_room::LeftRoom;
use super::super::room_rx::RoomRx;
use super::super::room_state::RoomState;
use super::super::selected_room::SelectedRoom;

pub fn poll_room(
    room_rx: Res<RoomRx>,
    mut active: ResMut<ActiveRoom>,
    mut selected: ResMut<SelectedRoom>,
    left: Res<LeftRoom>,
    mut pending: ResMut<JoinPending>,
    mut gate: ResMut<JoinGate>,
    mut clock: ResMut<JoinClock>,
    mut lobby: ResMut<roster::Lobby>,
    mut next: ResMut<NextState<RoomState>>,
) {
    let room = room_rx
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|rx| rx.borrow().clone()));
    let Some(room) = room else {
        return;
    };
    if Some(room.as_str()) == left.as_deref() {
        return;
    }
    if active.as_deref() == Some(room.as_str()) {
        return;
    }
    **active = Some(room.clone());
    **selected = Some(room.clone());
    *lobby = roster::Lobby(room.clone());
    **pending = Some(room);
    gate.expected = None;
    gate.committed = false;
    clock.clicked_at = Some(std::time::Instant::now());
    clock.last_new_peer = None;
    next.set(RoomState::InRoom);
}

#[cfg(test)]
mod tests {
    use super::poll_room;
    use crate::discovery;
    use crate::roster;
    use bevy::prelude::*;
    use std::sync::Mutex;

    #[test]
    fn test_usage() {
        let (tx, rx) = tokio::sync::watch::channel(None::<String>);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<discovery::RoomState>();
        app.insert_resource(discovery::RoomRx(Mutex::new(Some(rx))));
        app.insert_resource(discovery::ActiveRoom::default());
        app.insert_resource(discovery::SelectedRoom::default());
        app.insert_resource(discovery::LeftRoom::default());
        app.insert_resource(discovery::JoinPending::default());
        app.insert_resource(discovery::JoinGate::default());
        app.insert_resource(discovery::JoinClock::default());
        app.insert_resource(roster::Lobby::default());
        app.add_systems(Update, poll_room);
        app.update();
        assert!(app.world().resource::<discovery::ActiveRoom>().is_none());
        tx.send_replace(Some("room-auto".to_string()));
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<discovery::ActiveRoom>().as_deref(),
            Some("room-auto")
        );
        assert_eq!(
            app.world().resource::<roster::Lobby>().as_str(),
            "room-auto"
        );
        let state = app.world().resource::<State<discovery::RoomState>>();
        assert_eq!(**state, discovery::RoomState::InRoom);
    }
}
