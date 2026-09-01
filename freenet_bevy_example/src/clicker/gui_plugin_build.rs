use bevy::app::App;
use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;

use crate::clicker;

pub fn build(app: &mut App) {
    if !app.is_plugin_added::<TweeningPlugin>() {
        app.add_plugins(TweeningPlugin);
    }
    app.add_systems(Startup, clicker::bevy_systems::spawn_ui);
    app.add_systems(
        Update,
        (
            clicker::bevy_systems::handle_increment_click,
            clicker::bevy_systems::update_counter_ui,
            clicker::bevy_systems::update_status_bubble_ui,
            clicker::bevy_systems::update_status_tooltip_hover,
            clicker::bevy_systems::update_message_log_ui,
            clicker::bevy_systems::despawn_pending,
        ),
    );
}
// no test_usage necessary
