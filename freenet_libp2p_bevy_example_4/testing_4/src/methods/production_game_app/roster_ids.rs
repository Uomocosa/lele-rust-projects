use freenet_libp2p_bevy_example_4_lib::boxes;
use freenet_libp2p_bevy_example_4_lib::roster;

use crate::structs;

pub fn roster_ids(this: &structs::ProductionGameApp) -> Vec<boxes::PlayerId> {
    this.app
        .world()
        .resource::<roster::Roster>()
        .keys()
        .cloned()
        .collect()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
