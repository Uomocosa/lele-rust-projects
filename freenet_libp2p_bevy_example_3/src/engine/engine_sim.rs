use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use bevy_lele_rollback_plugin_1::Simulation;

use crate::engine;

impl Simulation for engine::Engine {
    type State = engine::EngineSimState;
    type Input = (engine::PlayerId, engine::Action);

    fn step(&mut self, tick: u64, inputs: &[Self::Input]) {
        let _ = super::engine_step::step(self, tick, inputs);
    }

    fn snapshot(&self) -> Self::State {
        let mut bodies = std::collections::BTreeMap::new();
        let world = self.app.world();
        for (&pid, entity) in &self.entities {
            let Ok(eref) = world.get_entity(*entity) else {
                continue;
            };
            let pos = eref.get::<Position>();
            let vel = eref.get::<LinearVelocity>();
            if let (Some(pos), Some(vel)) = (pos, vel) {
                bodies.insert(pid, (pos.0.x, pos.0.y, vel.x, vel.y));
            }
        }
        engine::EngineSimState {
            tick: self.tick,
            bodies,
        }
    }

    fn restore(&mut self, state: Self::State) {
        self.tick = state.tick;
        let world = self.app.world_mut();
        for (pid, entity) in &self.entities {
            let Some(body) = state.bodies.get(pid) else {
                continue;
            };
            let Ok(mut em) = world.get_entity_mut(*entity) else {
                continue;
            };
            if let Some(mut pos) = em.get_mut::<Position>() {
                pos.0 = Vec2::new(body.0, body.1);
            }
            if let Some(mut vel) = em.get_mut::<LinearVelocity>() {
                vel.x = body.2;
                vel.y = body.3;
            }
        }
    }

    fn hash(&self) -> u64 {
        let mut bodies = std::collections::BTreeMap::new();
        let world = self.app.world();
        for (&pid, entity) in &self.entities {
            let Ok(eref) = world.get_entity(*entity) else {
                continue;
            };
            if let Some(pos) = eref.get::<Position>() {
                bodies.insert(pid, (pos.0.x, pos.0.y));
            }
        }
        engine::hash_snapshot(&engine::Snapshot {
            tick: self.tick,
            bodies,
        })
    }
}

#[cfg(test)]
mod tests {
    use bevy_lele_rollback_plugin_1::Simulation;

    use crate::engine;

    #[test]
    fn test_usage() {
        let mut engine = engine::Engine::new();
        engine.spawn_player([1; 32]);
        Simulation::step(&mut engine, 1, &[([1; 32], engine::Action::default())]);
        let hash1 = engine.hash();
        let state1 = engine.snapshot();
        Simulation::step(&mut engine, 2, &[([1; 32], engine::Action::default())]);
        let hash2 = engine.hash();
        engine.restore(state1);
        assert_eq!(
            engine.hash(),
            hash1,
            "restore reproduces the earlier tick exactly"
        );
        Simulation::step(&mut engine, 2, &[([1; 32], engine::Action::default())]);
        assert_eq!(
            engine.hash(),
            hash2,
            "re-stepping after a restore reproduces the original trajectory"
        );
    }
}
