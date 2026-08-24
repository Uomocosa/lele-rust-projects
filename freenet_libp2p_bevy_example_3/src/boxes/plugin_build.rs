use bevy::prelude::*;

use crate::boxes;
use crate::engine;
use crate::netcode;

pub fn build(plugin: &boxes::Plugin, app: &mut App) {
    let sim: boxes::SimState = Default::default();
    app.insert_resource(**plugin)
        .insert_resource(engine::spawn_engine())
        .insert_resource(boxes::LatestSnapshot::default())
        .insert_resource(boxes::PredictedSnapshot::default())
        .insert_resource(boxes::NetcodeLockstep(netcode::Lockstep::new(vec![])))
        .insert_resource(sim)
        .add_systems(Startup, boxes::bevy_systems::setup)
        .add_systems(
            FixedUpdate,
            (
                boxes::bevy_systems::sync_engine_players,
                boxes::bevy_systems::netcode_tick,
                boxes::bevy_systems::render_snapshots,
            )
                .chain(),
        );
}

// no test_usage necessary — exercised by plugin.rs test_usage
