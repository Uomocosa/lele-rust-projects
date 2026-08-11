use bevy_freenet::roster;

use crate::structs::production_game_app::ProductionGameApp;

pub fn roster_len(this: &mut ProductionGameApp) -> usize {
    this.app.world().resource::<roster::Roster>().len()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
