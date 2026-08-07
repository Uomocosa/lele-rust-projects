use bevy::app::App;
use bevy::prelude::*;

use crate::clicker;

pub fn build(app: &mut App) {
    app.add_systems(Startup, clicker::bevy_systems::spawn_ui);
    app.add_systems(
        Update,
        (
            clicker::bevy_systems::handle_increment_click,
            clicker::bevy_systems::update_counter_ui,
        ),
    );
}
// no test_usage necessary
