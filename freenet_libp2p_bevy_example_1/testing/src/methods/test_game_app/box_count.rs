use freenet_libp2p_bevy_example_1_lib::boxes;

use crate::structs::test_game_app::TestGameApp;

pub fn box_count(this: &mut TestGameApp) -> usize {
    let mut query = this.app.world_mut().query::<&boxes::Player>();
    query.iter(this.app.world()).count()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
