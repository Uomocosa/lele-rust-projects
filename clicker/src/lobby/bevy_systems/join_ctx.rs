use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::roster;

use crate::clicker;
use crate::lobby;

#[derive(SystemParam)]
pub struct JoinCtx<'w> {
    pub(crate) active: ResMut<'w, clicker::ActiveLobby>,
    pub(crate) selected: ResMut<'w, lobby::SelectedRoom>,
    pub(crate) left: ResMut<'w, lobby::LeftRoom>,
    pub(crate) roster_lobby: ResMut<'w, roster::Lobby>,
    pub(crate) pending: ResMut<'w, lobby::JoinPending>,
    pub(crate) gate: ResMut<'w, lobby::JoinGate>,
    pub(crate) clock: ResMut<'w, lobby::JoinClock>,
    pub(crate) live: Res<'w, lobby::DirectoryLive>,
}

#[rustfmt::skip]
impl JoinCtx<'_> {
    pub(crate) fn rooms(&mut self) -> lobby::JoinRooms<'_> { super::join_ctx_rooms::rooms(self) }
}
// no test_usage necessary
