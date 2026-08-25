use avian2d::prelude::{LinearVelocity, Position};

use crate::engine;

pub fn restore(engine: &mut engine::Engine, state: &engine::EngineSimState) {
    engine.tick = state.tick;
    let mut query = engine
        .app
        .world_mut()
        .query::<(&engine::Player, &mut Position, &mut LinearVelocity)>();
    for (pid, mut position, mut velocity) in query.iter_mut(engine.app.world_mut()) {
        if let Some(body) = state.bodies.get(&pid.0) {
            position.0.x = body.0;
            position.0.y = body.1;
            velocity.x = body.2;
            velocity.y = body.3;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::restore;

    #[test]
    fn test_usage() {
        let mut engine = engine::Engine::new();
        engine.spawn_player([1; 32]);
        let mut bodies = std::collections::BTreeMap::new();
        bodies.insert([1; 32], (5.0, 10.0, 1.0, 2.0));
        let state = engine::EngineSimState { tick: 42, bodies };
        restore(&mut engine, &state);
        assert_eq!(engine.tick, 42);
        let pos = engine.player_position([1; 32]).unwrap();
        assert_eq!(pos.0, 5.0);
        assert_eq!(pos.1, 10.0);
    }
}
