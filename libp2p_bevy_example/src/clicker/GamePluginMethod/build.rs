use bevy::prelude::*;

use crate::clicker::system;
use crate::clicker::GamePlugin;

pub fn build(_plugin: &GamePlugin, app: &mut App) {
    app.add_systems(Update, (system::detect_click, system::update_counter))
        .add_systems(Update, system::handle_player_join)
        .add_systems(Update, system::handle_player_leave);
}

#[cfg(test)]
mod tests {
    use crate::clicker::GamePlugin;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(GamePlugin);
    }
}
