use bevy::prelude::*;

use crate::clicker;

pub fn build(app: &mut App) {
    app.add_systems(Startup, clicker::bevy_systems::setup)
        .add_systems(
            Update,
            (
                clicker::bevy_systems::spawn_on_join,
                clicker::bevy_systems::despawn_on_leave,
                clicker::bevy_systems::detect_click,
                clicker::bevy_systems::apply_delta,
                clicker::bevy_systems::render,
            ),
        );
}
// no test_usage necessary — exercised by integration tests
