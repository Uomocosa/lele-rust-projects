use freenet_libp2p_bevy_example_1_lib::boxes;
use freenet_libp2p_bevy_example_1_lib::roster;

use crate::structs::production_game_app::ProductionGameApp;

pub fn roster_ids(this: &ProductionGameApp) -> Vec<boxes::PlayerId> {
    this.app
        .world()
        .resource::<roster::Roster>()
        .keys()
        .cloned()
        .collect()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
