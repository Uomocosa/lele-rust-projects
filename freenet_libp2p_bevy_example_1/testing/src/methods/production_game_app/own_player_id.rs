use freenet_libp2p_bevy_example_1_lib::boxes;

use crate::structs::production_game_app::ProductionGameApp;

pub fn own_player_id(this: &ProductionGameApp) -> boxes::PlayerId {
    **this.app.world().resource::<boxes::Config>()
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
