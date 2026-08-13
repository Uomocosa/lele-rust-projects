use bevy::prelude::*;
use freenet_libp2p_bevy_example_1_lib::boxes;

use crate::structs::test_game_app::TestGameApp;

pub fn box_spawns(this: &mut TestGameApp) -> Vec<(boxes::PlayerId, Vec2, bool)> {
    let mut query = this
        .app
        .world_mut()
        .query::<(&boxes::Player, &Transform, Option<&boxes::LocalPlayer>)>();
    let mut spawns: Vec<(boxes::PlayerId, Vec2, bool)> = query
        .iter(this.app.world())
        .map(|(player, transform, local)| {
            (**player, transform.translation.truncate(), local.is_some())
        })
        .collect();
    spawns.sort_by_key(|(id, _, _)| *id);
    spawns
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
