use bevy::prelude::*;

use super::bevy_systems;
use super::p2p_room_discovery_ui_plugin::P2PRoomDiscoveryUiPlugin;
use super::room_state::RoomState;

pub fn build(_plugin: &P2PRoomDiscoveryUiPlugin, app: &mut App) {
    app.add_systems(
        Update,
        (
            bevy_systems::show_menu,
            bevy_systems::create_button,
            bevy_systems::join_button,
            bevy_systems::join_feedback,
        )
            .run_if(in_state(RoomState::Browsing).or_else(in_state(RoomState::Joining))),
    )
    .add_systems(
        Update,
        (bevy_systems::clear_pending, bevy_systems::leave_button)
            .run_if(in_state(RoomState::InRoom)),
    )
    .add_systems(OnEnter(RoomState::InRoom), bevy_systems::spawn_leave)
    .add_systems(OnExit(RoomState::Browsing), bevy_systems::despawn_menu)
    .add_systems(OnEnter(RoomState::Browsing), bevy_systems::despawn_leave);
}
// no test_usage necessary
