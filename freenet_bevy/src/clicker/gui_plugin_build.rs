use bevy::app::App;
use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.add_systems(Startup, crate::clicker::systems::spawn_ui::spawn_ui);
    app.add_systems(
        Update,
        (
            crate::clicker::systems::handle_increment_click::handle_increment_click,
            crate::clicker::systems::update_counter_ui::update_counter_ui,
        ),
    );
}
