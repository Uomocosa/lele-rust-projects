use avian2d::prelude::{LinearVelocity, Position};

use crate::engine;

pub fn sim_state(engine: &mut engine::Engine) -> engine::EngineSimState {
    let mut bodies = std::collections::BTreeMap::new();
    let mut query = engine
        .app
        .world_mut()
        .query::<(&engine::Player, &Position, &LinearVelocity)>();
    for (pid, position, velocity) in query.iter(engine.app.world()) {
        bodies.insert(pid.0, (position.0.x, position.0.y, velocity.x, velocity.y));
    }
    engine::EngineSimState {
        tick: engine.tick,
        bodies,
    }
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::sim_state;

    #[test]
    fn test_usage() {
        let mut engine = engine::Engine::new();
        engine.spawn_player([1; 32]);
        let state = sim_state(&mut engine);
        assert_eq!(state.tick, 0);
        assert_eq!(state.bodies.len(), 1);
        let (_, body) = state.bodies.iter().next().unwrap();
        assert_eq!(body.3, 0.0);
    }
}
