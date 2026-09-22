use atomic_delegate_macros::atomic_delegate;
use bevy::prelude::{App, Plugin};

use super::global_counter::GlobalCounter;

pub struct GamePlugin;

#[atomic_delegate]
impl GamePlugin {
    pub fn new() -> Self {}
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalCounter>();
    }
}

#[cfg(test)]
mod tests {
    use super::GamePlugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(GamePlugin::new());
        app.update();
    }
}
