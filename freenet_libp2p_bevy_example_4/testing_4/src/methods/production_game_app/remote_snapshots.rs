use bevy::math::Vec2;
use freenet_libp2p_bevy_example_4_lib::boxes;

use crate::structs;

pub fn remote_snapshots(
    this: &mut structs::ProductionGameApp,
) -> Vec<(boxes::PlayerId, Vec2, u64)> {
    let own = **this.app.world().resource::<boxes::Config>();
    let snapshot = this.app.world().resource::<boxes::LatestSnapshot>();
    let Some(snapshot) = snapshot.as_ref() else {
        return Vec::new();
    };
    let mut snaps: Vec<(boxes::PlayerId, Vec2, u64)> = Vec::new();
    for (id, (x, y)) in &snapshot.bodies {
        if *id == own {
            continue;
        }
        snaps.push((*id, Vec2::new(*x, *y), snapshot.tick));
    }
    snaps.sort_by_key(|(id, _, _)| *id);
    snaps
}
// no test_usage necessary - needs a live embedded freenet node, exercised by tests/
