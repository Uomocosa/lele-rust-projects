#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::create_marker::CreateMarker;
use super::super::directory_live::DirectoryLive;
use super::super::join_pending::JoinPending;
use super::super::loading_root::LoadingRoot;
use super::super::room_request::RoomRequest;
use super::super::room_state::RoomState;

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
    if !pressed || pending.is_some() {
        return;
    }
    let room = format!("room-{}", epoch_secs());
    requests.write(RoomRequest::Join(room.clone()));
    commands.spawn((LoadingRoot, Text::new(format!("Joining {room}..."))));
    **pending = Some(room);
    next.set(RoomState::Joining);
}

// needed helper: seconds since the unix epoch for generated room names
fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}
// no test_usage necessary
