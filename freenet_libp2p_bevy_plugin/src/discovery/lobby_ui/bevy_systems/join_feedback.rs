#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::super::session::join_pending::JoinPending;
use super::super::room_button::RoomButton;

pub fn join_feedback(
    pending: Res<JoinPending>,
    mut buttons: Query<(&RoomButton, &mut BackgroundColor)>,
) {
    let Some(room) = (**pending).clone() else {
        return;
    };
    for (button, mut color) in &mut buttons {
        if **button == room {
            *color = BackgroundColor(Color::srgb(0.2, 0.45, 0.7));
        } else {
            *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
        }
    }
}
// no test_usage necessary
