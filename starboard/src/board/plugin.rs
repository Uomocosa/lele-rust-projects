use bevy::prelude::*;

use crate::board::bevy_systems;

pub struct SharedBlackboardPlugin;

impl Plugin for SharedBlackboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, bevy_systems::handle_click::handle_click);
    }
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
