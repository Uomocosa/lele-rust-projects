use freenet_libp2p_bevy_example_4_lib::roster;

use crate::structs;

pub fn roster_len(this: &mut structs::TestGameApp) -> usize {
    this.app.world().resource::<roster::Roster>().len()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
