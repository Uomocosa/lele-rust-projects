#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use crate::roster;

use super::super::super::session::active_room::ActiveRoom;
use super::super::super::session::confirm_joined::confirm_joined;
use super::super::super::session::is_joinable::is_joinable;
use super::super::super::session::join_clock::JoinClock;
use super::super::super::session::join_gate::JoinGate;
use super::super::super::session::join_pending::JoinPending;
use super::super::super::session::left_room::LeftRoom;
use super::super::super::session::room_rx::RoomRx;
use super::super::super::session::room_state::RoomState;
use super::super::super::session::selected_room::SelectedRoom;

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
    if !is_joinable(&left, &active, &room) {
        return;
    }
    confirm_joined(
        &mut active,
        &mut selected,
        &mut pending,
        &mut gate,
        &mut clock,
        &mut lobby,
        room,
    );
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
        app.init_state::<discovery::session::RoomState>();
        app.insert_resource(discovery::session::RoomRx(Mutex::new(Some(rx))));
        app.insert_resource(discovery::session::ActiveRoom::default());
        app.insert_resource(discovery::session::SelectedRoom::default());
        app.insert_resource(discovery::session::LeftRoom::default());
        app.insert_resource(discovery::session::JoinPending::default());
        app.insert_resource(discovery::session::JoinGate::default());
        app.insert_resource(discovery::session::JoinClock::default());
        app.insert_resource(roster::Lobby::default());
        app.add_systems(Update, poll_room);
        app.update();
        assert!(
            app.world()
                .resource::<discovery::session::ActiveRoom>()
                .is_none()
        );
        tx.send_replace(Some("room-auto".to_string()));
        app.update();
        app.update();
        assert_eq!(
            app.world()
                .resource::<discovery::session::ActiveRoom>()
                .as_deref(),
            Some("room-auto")
        );
        assert_eq!(
            app.world().resource::<roster::Lobby>().as_str(),
            "room-auto"
        );
        let state = app
            .world()
            .resource::<State<discovery::session::RoomState>>();
        assert_eq!(**state, discovery::session::RoomState::InRoom);
    }
}
