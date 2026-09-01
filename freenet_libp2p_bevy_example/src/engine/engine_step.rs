use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::engine;

pub fn step(
    engine: &mut engine::Engine,
    tick: u64,
    actions: &[(engine::PlayerId, engine::Action)],
) -> Result<engine::Snapshot, engine::Error> {
    let pending = actions
        .iter()
        .copied()
        .collect::<std::collections::BTreeMap<_, _>>();
    engine
        .app
        .world_mut()
        .insert_resource(engine::PendingActions(pending));
    engine.app.world_mut().insert_resource(engine::Tick(tick));
    engine.tick = tick;

    engine.app.update();

    let mut bodies = std::collections::BTreeMap::new();
    let mut query = engine
        .app
        .world_mut()
        .query::<(&engine::Player, &Position)>();
    for (pid, position) in query.iter(engine.app.world()) {
        bodies.insert(pid.0, (position.0.x, position.0.y));
    }

    Ok(engine::Snapshot { tick, bodies })
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::step;

    #[test]
    fn test_usage() {
        let mut engine = engine::Engine::new();
        engine.spawn_player([1; 32]);
        let snapshot = step(&mut engine, 0, &[]).unwrap();
        assert_eq!(snapshot.tick, 0);
        assert_eq!(snapshot.bodies.len(), 1);
    }
}
