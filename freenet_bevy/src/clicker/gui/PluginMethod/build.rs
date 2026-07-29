use bevy::app::App;
use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.add_systems(Startup, crate::clicker::gui::system::spawn_ui::spawn_ui);
    app.add_systems(
        Update,
        (
            crate::clicker::gui::system::increment_button::increment_button,
            crate::clicker::gui::system::update_counter_ui::update_counter_ui,
        ),
    );
}
