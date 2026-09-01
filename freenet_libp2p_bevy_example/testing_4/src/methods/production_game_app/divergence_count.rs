use freenet_libp2p_bevy_example_lib::boxes;

use crate::structs;

pub fn divergence_count(this: &structs::ProductionGameApp) -> u64 {
    this.app
        .world()
        .resource::<boxes::SimState>()
        .divergence_count
}

// no test_usage necessary - needs a live embedded freenet node, exercised by tests/
