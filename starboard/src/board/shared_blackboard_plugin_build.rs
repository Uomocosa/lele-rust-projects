use bevy::prelude::*;

use super::shared_blackboard_plugin::SharedBlackboardPlugin;
use crate::board;

pub fn build(_plugin: &SharedBlackboardPlugin, app: &mut App) {
    app.add_systems(Update, board::bevy_systems::handle_click::handle_click);
}

#[cfg(test)]
mod tests {
    use super::build;
    use bevy::prelude::*;

    use super::SharedBlackboardPlugin;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ButtonInput::<MouseButton>::default());
        build(&SharedBlackboardPlugin, &mut app);
        app.update();
    }
}
