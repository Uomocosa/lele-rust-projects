use freenet_libp2p_bevy_example_3_lib::boxes;

use crate::structs;

pub fn sim_clock(this: &mut structs::ProductionGameApp) -> u64 {
    this.app.world().resource::<boxes::SimState>().clock
}
// no test_usage necessary - needs a live embedded freenet node, exercised by tests/
