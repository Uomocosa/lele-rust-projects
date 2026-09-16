use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, roster};

use crate::clicker;
use crate::lobby;

#[derive(SystemParam)]
pub struct SpawnCtx<'w> {
    pub(crate) roster: Res<'w, roster::Roster>,
    pub(crate) lobby: Res<'w, clicker::ActiveLobby>,
    pub(crate) own: Res<'w, net_id::NetworkId>,
    pub(crate) pending: Res<'w, lobby::JoinPending>,
}

#[rustfmt::skip]
impl SpawnCtx<'_> {
    pub(crate) fn gate_open(&self) -> bool { super::spawn_ctx_gate_open::gate_open(self) }
}
// no test_usage necessary
