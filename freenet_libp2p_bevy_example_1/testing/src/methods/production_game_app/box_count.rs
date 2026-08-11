use bevy_freenet::boxes;

use crate::structs::production_game_app::ProductionGameApp;

pub fn box_count(this: &mut ProductionGameApp) -> usize {
    let mut query = this.app.world_mut().query::<&boxes::Player>();
    query.iter(this.app.world()).count()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
