use avian2d::prelude::{Collider, LinearVelocity, LockedAxes, RigidBody, SleepingDisabled};
use bevy::prelude::*;

use crate::engine;

pub fn spawn_player(engine: &mut engine::Engine, id: engine::PlayerId) {
    if engine.entities.contains_key(&id) {
        return;
    }
    let entity = engine
        .app
        .world_mut()
        .spawn((
            engine::Player(id),
            RigidBody::Dynamic,
            Collider::rectangle(engine::BOX_SIZE, engine::BOX_SIZE),
            LockedAxes::ROTATION_LOCKED,
            SleepingDisabled,
            LinearVelocity::ZERO,
            Transform::from_xyz(engine::spawn_x_for_player(id), engine::SPAWN_Y, 0.0),
        ))
        .id();
    engine.entities.insert(id, entity);
}

#[cfg(test)]
mod tests {
    use crate::engine;

    #[test]
    fn test_usage() {
        let mut engine = engine::Engine::new();
        engine.spawn_player([1; 32]);
        assert!(engine.entities.contains_key(&[1; 32]));
        engine.spawn_player([1; 32]);
        assert_eq!(engine.entities.len(), 1);
    }
}
