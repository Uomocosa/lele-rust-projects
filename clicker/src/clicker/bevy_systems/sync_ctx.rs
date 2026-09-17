use std::collections::HashSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::net_id;

use crate::clicker;
use crate::lobby;

use super::sync_ctx_snapshot_entries;

#[derive(SystemParam)]
pub struct SyncCtx<'w, 's> {
    pub(crate) gate: ResMut<'w, lobby::JoinGate>,
    pub(crate) tombstones: Res<'w, clicker::ScoreTombstones>,
    pub(crate) seen: Local<'s, HashSet<u64, std::hash::RandomState>>,
}

#[rustfmt::skip]
impl SyncCtx<'_, '_> {
    pub(crate) fn snapshot_entries(
        &mut self,
        targets: &Query<(&clicker::Owner, Option<&clicker::PlayerNo>, &mut clicker::ClickCounter)>,
    ) -> Vec<(net_id::NetworkId, i32)> { sync_ctx_snapshot_entries::snapshot_entries(self, targets) }
}
// no test_usage necessary
