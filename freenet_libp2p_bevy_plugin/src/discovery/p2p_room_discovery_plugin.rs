use atomic_delegate_macros::atomic_delegate;
use bevy::prelude::App;
use derive_more::Deref;

use super::config::Config;

#[derive(Deref)]
pub struct P2PRoomDiscoveryPlugin(pub Config);

impl P2PRoomDiscoveryPlugin {
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self(config)
    }
}

#[atomic_delegate]
impl P2PRoomDiscoveryPlugin {
    pub fn build_plugin(&self, app: &mut App) {}
}

impl bevy::prelude::Plugin for P2PRoomDiscoveryPlugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}
// no test_usage necessary
