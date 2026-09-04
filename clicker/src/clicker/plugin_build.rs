use crate::clicker;
use bevy::prelude::*;
pub fn build(_plugin: &clicker::Plugin, app: &mut App) {
    app.add_systems(Startup, clicker::bevy_systems::setup)
        .add_systems(Update, clicker::bevy_systems::detect_click)
        .add_systems(Update, clicker::bevy_systems::spawn_on_join)
        .add_systems(Update, clicker::bevy_systems::despawn_on_leave)
        .add_systems(Update, clicker::bevy_systems::apply_delta)
        .add_systems(Update, clicker::bevy_systems::render);
}
