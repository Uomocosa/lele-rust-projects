use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultUiPlugin;

#[atomic_delegates]
impl DefaultUiPlugin {
    pub fn build_plugin(&self, app: &mut App) {}
}

#[rustfmt::skip]
impl bevy::prelude::Plugin for DefaultUiPlugin {
    fn build(&self, app: &mut App) {
        self.build_plugin(app);
    }
}
// no test_usage necessary
