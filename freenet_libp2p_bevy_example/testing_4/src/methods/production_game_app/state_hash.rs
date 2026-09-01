use freenet_libp2p_bevy_example_lib::{boxes, engine};

use crate::structs;

pub fn state_hash(this: &structs::ProductionGameApp) -> u64 {
    let snapshot = this.app.world().resource::<boxes::LatestSnapshot>();
    let Some(snapshot) = snapshot.as_ref() else {
        return 0;
    };
    engine::hash_snapshot(snapshot)
}

pub fn own_box_position(this: &structs::ProductionGameApp) -> Option<bevy::math::Vec2> {
    let own = **this.app.world().resource::<boxes::Config>();
    let snapshot = this.app.world().resource::<boxes::LatestSnapshot>();
    snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.bodies.get(&own))
        .map(|(x, y)| bevy::math::Vec2::new(*x, *y))
}
// no test_usage necessary - needs a live embedded freenet node, exercised by tests/
