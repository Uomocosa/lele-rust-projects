use std::collections::HashMap;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;

use super::leave_ctx_absence_secs;

#[derive(SystemParam)]
pub struct LeaveCtx<'w, 's> {
    pub(crate) lobby: Res<'w, clicker::ActiveLobby>,
    pub(crate) own: Res<'w, net_id::NetworkId>,
    pub(crate) tombstones: ResMut<'w, clicker::ScoreTombstones>,
    pub(crate) time: Res<'w, Time>,
    pub(crate) absent: Local<'s, HashMap<u64, f64, std::hash::RandomState>>,
}

#[rustfmt::skip]
impl LeaveCtx<'_, '_> {
    pub(crate) fn absence_secs(&mut self, owner: u64) -> f64 { leave_ctx_absence_secs::absence_secs(self, owner) }
}
// no test_usage necessary
