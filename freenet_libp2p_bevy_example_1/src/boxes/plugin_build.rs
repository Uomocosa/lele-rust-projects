use avian2d::prelude::{Gravity, PhysicsPlugins};
use bevy::prelude::*;

use super::plugin::Plugin;
use crate::boxes::bevy_systems;
use crate::boxes::constants;

pub fn build(_plugin: &Plugin, app: &mut App) {
    app.add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec2::NEG_Y * constants::GRAVITY))
        .add_systems(Startup, bevy_systems::setup)
        .add_systems(Update, bevy_systems::read_input);
}

// no test_usage necessary — exercised by plugin.rs test_usage
