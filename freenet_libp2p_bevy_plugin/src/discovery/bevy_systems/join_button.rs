#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;

use super::super::directory_live::DirectoryLive;
use super::super::join_pending::JoinPending;
use super::super::loading_root::LoadingRoot;
use super::super::room_button::RoomButton;
use super::super::room_request::RoomRequest;
use super::super::room_state::RoomState;

pub fn join_button(
    mut commands: Commands,
    triggers: Query<(&Interaction, &RoomButton), Changed<Interaction>>,
    live: Res<DirectoryLive>,
    mut requests: MessageWriter<RoomRequest>,
    mut pending: ResMut<JoinPending>,
    mut next: ResMut<NextState<RoomState>>,
) {
    if !**live {
        return;
    }
    let mut chosen: Option<String> = None;
    for (interaction, button) in &triggers {
        if *interaction == Interaction::Pressed {
            chosen = Some((**button).clone());
        }
    }
    let Some(room) = chosen else {
        return;
    };
    if room.is_empty() || pending.is_some() {
        return;
    }
    requests.write(RoomRequest::Join(room.clone()));
    commands.spawn((LoadingRoot, Text::new(format!("Joining {room}..."))));
    **pending = Some(room);
    next.set(RoomState::Joining);
}
// no test_usage necessary
