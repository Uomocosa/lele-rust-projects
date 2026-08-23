use std::collections::BTreeMap;
use std::time::Duration;

use avian2d::prelude::{PhysicsPlugins, PhysicsSystems};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::engine;

pub fn new() -> engine::Engine {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::transform::TransformPlugin,
        PhysicsPlugins::default(),
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / engine::TICKS_PER_SECOND as f64,
    )))
    .add_systems(Startup, engine::bevy_systems::setup_world)
    .add_systems(
        FixedPostUpdate,
        engine::bevy_systems::apply_pending_actions.in_set(PhysicsSystems::First),
    );
    app.insert_resource(engine::PendingActions::default());
    app.insert_resource(engine::Tick(0));
    app.finish();

    engine::Engine {
        app,
        entities: BTreeMap::new(),
        tick: 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;

    #[test]
    fn test_usage() {
        let engine = engine::Engine::new();
        assert_eq!(engine.tick, 0);
        assert!(engine.entities.is_empty());
    }
}
