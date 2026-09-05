use bevy::prelude::*;

use super::shared_blackboard_plugin_build;

pub struct SharedBlackboardPlugin;

#[rustfmt::skip]
impl Plugin for SharedBlackboardPlugin {
    fn build(&self, app: &mut App) { shared_blackboard_plugin_build::build(self, app) }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::SharedBlackboardPlugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ButtonInput::<MouseButton>::default());
        app.add_plugins(SharedBlackboardPlugin);
        app.update();
    }
}
