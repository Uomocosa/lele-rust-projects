use bevy_freenet::roster;

use crate::structs::test_game_app::TestGameApp;

pub fn roster_len(this: &mut TestGameApp) -> usize {
    this.app.world().resource::<roster::Roster>().len()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
