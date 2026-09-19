use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use freenet_libp2p_bevy_plugin::{net_id, p2p};

use crate::clicker;
use crate::lobby;

#[derive(SystemParam)]
pub struct ResolveCtx<'w, 's> {
    pub(crate) gate: Res<'w, lobby::JoinGate>,
    pub(crate) events: ResMut<'w, p2p::Events<clicker::CursorMsg>>,
    pub(crate) peers: Query<
        'w,
        's,
        (
            Entity,
            &'static clicker::Owner,
            Option<&'static clicker::PlayerNo>,
            Option<&'static clicker::PendingReveal>,
        ),
    >,
    pub(crate) spots: Query<
        'w,
        's,
        (
            &'static clicker::Owner,
            &'static mut Transform,
            &'static mut clicker::TargetPos,
        ),
        With<clicker::CursorIcon>,
    >,
    pub(crate) counters: Query<'w, 's, &'static mut clicker::ClickCounter>,
    pub(crate) commands: Commands<'w, 's>,
    pub(crate) materials: ResMut<'w, Assets<ColorMaterial>>,
    pub(crate) tombstones: ResMut<'w, clicker::ScoreTombstones>,
    pub(crate) lobby: Res<'w, clicker::ActiveLobby>,
    pub(crate) own: Res<'w, net_id::NetworkId>,
}

#[rustfmt::skip]
impl ResolveCtx<'_, '_> {
    #[must_use]
    pub fn topic(&self) -> String { clicker::pos_topic(&self.lobby) }
}
// no test_usage necessary
