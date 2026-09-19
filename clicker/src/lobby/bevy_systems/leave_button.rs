use bevy::prelude::*;

use crate::clicker;
use crate::lobby;

pub fn leave_button(
    triggers: Query<&Interaction, (Changed<Interaction>, With<lobby::bevy_systems::LeaveMarker>)>,
    mut rooms: lobby::bevy_systems::JoinCtx,
    global: ResMut<clicker::GlobalCounter>,
    pending: ResMut<clicker::PendingClicks>,
    tombstones: ResMut<clicker::ScoreTombstones>,
    next: ResMut<NextState<lobby::AppState>>,
) {
    let pressed = triggers
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed {
        return;
    }
    let global = global.into_inner();
    let pending = pending.into_inner();
    let tombstones = tombstones.into_inner();
    let next = next.into_inner();
    lobby::leave_room(
        &mut rooms.active,
        global,
        pending,
        tombstones,
        &mut rooms.selected,
        &mut rooms.left,
    );
    **rooms.pending = None;
    next.set(lobby::AppState::Menu);
}

#[cfg(test)]
mod tests {
    use super::leave_button;
    use crate::clicker;
    use crate::lobby;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.world_mut()
            .resource_mut::<NextState<lobby::AppState>>()
            .set(lobby::AppState::InRoom);
        app.update();
        app.insert_resource(clicker::ActiveLobby("room-a".to_string()));
        app.insert_resource(clicker::GlobalCounter(9));
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(lobby::JoinPending::default());
        app.insert_resource(lobby::SelectedRoom(Some("room-a".to_string())));
        app.insert_resource(lobby::LeftRoom::default());
        app.insert_resource(freenet_libp2p_bevy_plugin::roster::Lobby::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::DirectoryLive(true));
        let (_tx, rx) = tokio::sync::watch::channel(Some("room-a".to_string()));
        app.insert_resource(lobby::RoomRx(std::sync::Mutex::new(Some(rx))));
        app.world_mut()
            .resource_mut::<clicker::ScoreTombstones>()
            .keep(2, 5);
        app.world_mut().spawn((
            lobby::bevy_systems::LeaveRoot,
            lobby::bevy_systems::LeaveMarker,
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, leave_button);
        app.update();
        app.update();
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(active, &clicker::ActiveLobby::default());
        assert_eq!(**app.world().resource::<clicker::GlobalCounter>(), 0);
        assert!(
            app.world()
                .resource::<clicker::ScoreTombstones>()
                .is_empty()
        );
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::Menu);
    }

    #[test]
    fn leave_during_loading_clears_overlay() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.world_mut()
            .resource_mut::<NextState<lobby::AppState>>()
            .set(lobby::AppState::InRoom);
        app.update();
        app.insert_resource(clicker::ActiveLobby("room-a".to_string()));
        app.insert_resource(clicker::GlobalCounter::default());
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(lobby::JoinPending(Some("room-a".to_string())));
        app.insert_resource(lobby::SelectedRoom(Some("room-a".to_string())));
        app.insert_resource(lobby::LeftRoom::default());
        app.insert_resource(freenet_libp2p_bevy_plugin::roster::Lobby::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::DirectoryLive(true));
        let (_tx, rx) = tokio::sync::watch::channel(Some("room-a".to_string()));
        app.insert_resource(lobby::RoomRx(std::sync::Mutex::new(Some(rx))));
        let overlay = app.world_mut().spawn(lobby::bevy_systems::LoadingRoot).id();
        app.world_mut().spawn((
            lobby::bevy_systems::LeaveRoot,
            lobby::bevy_systems::LeaveMarker,
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(Update, leave_button);
        app.add_systems(
            Update,
            lobby::bevy_systems::despawn_leave.run_if(in_state(lobby::AppState::Menu)),
        );
        app.update();
        app.update();
        app.update();
        assert!(
            app.world().resource::<lobby::JoinPending>().is_none(),
            "leave aborts the pending join"
        );
        assert!(
            app.world().get_entity(overlay).is_err(),
            "leave removes the loading overlay"
        );
    }

    #[test]
    fn leave_sticks_across_apply_room() {
        use freenet_libp2p_bevy_plugin::p2p;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<lobby::AppState>();
        app.world_mut()
            .resource_mut::<NextState<lobby::AppState>>()
            .set(lobby::AppState::InRoom);
        app.update();
        app.insert_resource(clicker::ActiveLobby("room-a".to_string()));
        app.insert_resource(clicker::GlobalCounter(9));
        app.insert_resource(clicker::PendingClicks::default());
        app.insert_resource(clicker::ScoreTombstones::default());
        app.insert_resource(lobby::JoinPending::default());
        app.insert_resource(lobby::SelectedRoom(Some("room-a".to_string())));
        app.insert_resource(lobby::LeftRoom::default());
        let (_tx, rx) = tokio::sync::watch::channel(Some("room-a".to_string()));
        app.insert_resource(lobby::RoomRx(std::sync::Mutex::new(Some(rx))));
        app.insert_resource(freenet_libp2p_bevy_plugin::roster::Lobby::default());
        app.insert_resource(lobby::JoinClock::default());
        app.insert_resource(lobby::JoinGate::default());
        app.insert_resource(lobby::DirectoryLive(true));
        app.insert_resource(p2p::Commands::<clicker::CursorMsg>::default());
        app.world_mut().spawn((
            lobby::bevy_systems::LeaveRoot,
            lobby::bevy_systems::LeaveMarker,
            Button,
            Interaction::Pressed,
        ));
        app.add_systems(
            Update,
            (
                leave_button,
                lobby::bevy_systems::apply_room,
                lobby::bevy_systems::despawn_leave.run_if(in_state(lobby::AppState::Menu)),
            )
                .chain(),
        );
        for _ in 0..5 {
            app.update();
        }
        let state = app.world().resource::<State<lobby::AppState>>();
        assert_eq!(**state, lobby::AppState::Menu);
        let active = app.world().resource::<clicker::ActiveLobby>();
        assert_eq!(active, &clicker::ActiveLobby::default());
    }
}
