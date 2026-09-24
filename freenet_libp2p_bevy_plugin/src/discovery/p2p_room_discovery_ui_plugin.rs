use bevy::prelude::App;

use super::p2p_room_discovery_ui_plugin_build;

#[derive(Default)]
pub struct P2PRoomDiscoveryUiPlugin;

impl P2PRoomDiscoveryUiPlugin {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl bevy::prelude::Plugin for P2PRoomDiscoveryUiPlugin {
    fn build(&self, app: &mut App) {
        p2p_room_discovery_ui_plugin_build::build(self, app);
    }
}
// no test_usage necessary
