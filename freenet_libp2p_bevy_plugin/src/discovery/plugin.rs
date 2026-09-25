use atomic_delegate_macros::atomic_delegate;
use bevy::prelude::App;
use derive_more::Deref;

use super::config::Config;

#[derive(Deref)]
pub struct Plugin(pub Config);

impl Plugin {
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self(config)
    }
}

#[atomic_delegate]
impl Plugin {
    pub fn build_plugin(&self, app: &mut App) {}
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}
// no test_usage necessary
