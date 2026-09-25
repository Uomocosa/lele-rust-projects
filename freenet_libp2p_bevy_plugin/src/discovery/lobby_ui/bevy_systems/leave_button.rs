#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::super::session::active_room::ActiveRoom;
use super::super::super::session::join_gate::JoinGate;
use super::super::super::session::join_pending::JoinPending;
use super::super::super::session::leave_rooms::leave_rooms;
use super::super::super::session::left_room::LeftRoom;
use super::super::super::session::room_state::RoomState;
use super::super::super::session::selected_room::SelectedRoom;
use super::super::leave_marker::LeaveMarker;

pub fn leave_button(
    triggers: Query<&Interaction, (Changed<Interaction>, With<LeaveMarker>)>,
    mut active: ResMut<ActiveRoom>,
    mut selected: ResMut<SelectedRoom>,
    mut left: ResMut<LeftRoom>,
    mut pending: ResMut<JoinPending>,
    mut gate: ResMut<JoinGate>,
    mut next: ResMut<NextState<RoomState>>,
) {
    let pressed = triggers
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed {
        return;
    }
    leave_rooms(
        &mut active,
        &mut selected,
        &mut left,
        &mut pending,
        &mut gate,
    );
    next.set(RoomState::Browsing);
}
// no test_usage necessary
