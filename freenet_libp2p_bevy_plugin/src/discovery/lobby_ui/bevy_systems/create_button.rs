#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::super::link::epoch_secs::epoch_secs;
use super::super::super::session::begin_join::begin_join;
use super::super::super::session::directory_live::DirectoryLive;
use super::super::super::session::join_pending::JoinPending;
use super::super::super::session::room_request::RoomRequest;
use super::super::super::session::room_state::RoomState;
use super::super::create_marker::CreateMarker;
use super::super::loading_root::LoadingRoot;

pub fn create_button(
    mut commands: Commands,
    triggers: Query<&Interaction, (Changed<Interaction>, With<CreateMarker>)>,
    live: Res<DirectoryLive>,
    mut requests: MessageWriter<RoomRequest>,
    mut pending: ResMut<JoinPending>,
    mut next: ResMut<NextState<RoomState>>,
) {
    if !**live {
        return;
    }
    let pressed = triggers
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed {
        return;
    }
    let room = format!("room-{}", epoch_secs());
    if !begin_join(&mut pending, room.clone()) {
        return;
    }
    requests.write(RoomRequest::Join(room.clone()));
    commands.spawn((LoadingRoot, Text::new(format!("Joining {room}..."))));
    next.set(RoomState::Joining);
}

// no test_usage necessary
