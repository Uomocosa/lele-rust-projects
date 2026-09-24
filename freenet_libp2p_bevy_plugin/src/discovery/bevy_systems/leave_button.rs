#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::active_room::ActiveRoom;
use super::super::join_gate::JoinGate;
use super::super::join_pending::JoinPending;
use super::super::leave_marker::LeaveMarker;
use super::super::left_room::LeftRoom;
use super::super::room_state::RoomState;
use super::super::selected_room::SelectedRoom;

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
    if let Some(room) = active.as_deref() {
        **left = Some(room.to_string());
    }
    **active = None;
    **selected = None;
    **pending = None;
    gate.expected = None;
    gate.committed = false;
    next.set(RoomState::Browsing);
}
// no test_usage necessary
