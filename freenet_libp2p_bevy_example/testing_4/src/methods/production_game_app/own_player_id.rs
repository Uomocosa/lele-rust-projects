use freenet_libp2p_bevy_example_lib::boxes;

use crate::structs;

pub fn own_player_id(this: &structs::ProductionGameApp) -> boxes::PlayerId {
    **this.app.world().resource::<boxes::Config>()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
