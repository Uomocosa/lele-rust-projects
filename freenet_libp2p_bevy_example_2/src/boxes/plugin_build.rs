use avian2d::prelude::{Gravity, PhysicsPlugins};
use bevy::prelude::*;

use crate::boxes;

pub fn build(plugin: &boxes::Plugin, app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::NEG_Y * boxes::constants::GRAVITY))
        .insert_resource(**plugin)
        .add_systems(Startup, boxes::bevy_systems::setup)
        .add_systems(Update, boxes::bevy_systems::read_input);
}

// no test_usage necessary — exercised by plugin.rs test_usage
