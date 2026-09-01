use freenet_libp2p_bevy_example_lib::boxes;

use crate::structs;

pub fn box_count(this: &mut structs::ProductionGameApp) -> usize {
    let mut query = this.app.world_mut().query::<&boxes::Player>();
    query.iter(this.app.world()).count()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
