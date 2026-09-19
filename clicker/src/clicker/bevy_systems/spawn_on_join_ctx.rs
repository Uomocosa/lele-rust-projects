use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{net_id, roster};

use super::spawn_on_join_ctx_held;
use crate::clicker;
use crate::lobby;

#[derive(SystemParam)]
pub struct SpawnOnJoinCtx<'w> {
    pub(crate) roster: Res<'w, roster::Roster>,
    pub(crate) lobby: Res<'w, clicker::ActiveLobby>,
    pub(crate) own: Res<'w, net_id::NetworkId>,
    pub(crate) gate: Res<'w, lobby::JoinGate>,
}

#[rustfmt::skip]
impl SpawnOnJoinCtx<'_> {
    #[must_use]
    pub fn held(&self, peer: &str, id: net_id::NetworkId) -> bool { spawn_on_join_ctx_held::held(*self.own, &self.gate.pending, &self.gate.absent, peer, id) }
}
// no test_usage necessary
