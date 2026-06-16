use bevy::prelude::*;

use crate::boxes::system;
use crate::boxes::GamePlugin;

pub fn build(_plugin: &GamePlugin, app: &mut App) {
    app.add_systems(FixedUpdate, system::collect)
        .add_systems(FixedUpdate, system::character_controller)
        .add_systems(FixedUpdate, system::sync_position)
        .add_systems(Update, system::handle_player_join)
        .add_systems(Update, system::handle_player_leave);
}

#[cfg(test)]
mod tests {
    use crate::boxes::GamePlugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(GamePlugin);
    }
}
