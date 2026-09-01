use avian2d::prelude::Position;

use crate::engine;

pub fn player_position(engine: &engine::Engine, id: engine::PlayerId) -> Option<(f32, f32)> {
    let entity = *engine.entities.get(&id)?;
    engine
        .app
        .world()
        .entity(entity)
        .get::<Position>()
        .map(|position| (position.0.x, position.0.y))
}

#[cfg(test)]
mod tests {
    use crate::engine;

    use super::player_position;

    #[test]
    fn test_usage() {
        let mut engine = engine::Engine::new();
        assert_eq!(player_position(&engine, [1; 32]), None);
        engine.spawn_player([1; 32]);
        assert!(player_position(&engine, [1; 32]).is_some());
    }
}
