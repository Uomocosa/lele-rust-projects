use bevy::math::Vec2;
use bevy::prelude::*;
use freenet_libp2p_bevy_example_2_lib::{boxes, p2p};

use crate::structs;

pub fn remote_snapshots(
    this: &mut structs::ProductionGameApp,
) -> Vec<(boxes::PlayerId, Vec2, u64)> {
    let mut query = this
        .app
        .world_mut()
        .query::<(&boxes::Player, &Transform, &p2p::RemoteTarget)>();
    let mut snaps: Vec<(boxes::PlayerId, Vec2, u64)> = query
        .iter(this.app.world())
        .map(|(player, transform, target)| {
            (
                **player,
                transform.translation.truncate(),
                target.sent_at_ms,
            )
        })
        .collect();
    snaps.sort_by_key(|(id, _, _)| *id);
    snaps
}
// no test_usage necessary — needs a live embedded freenet node, exercised by tests/
