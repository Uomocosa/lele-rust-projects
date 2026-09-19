use bevy::prelude::App;

use crate::clicker;
use crate::testing;

#[must_use]
pub fn counts_of(app: &mut App) -> testing::MeshCount {
    let mut out = testing::MeshCount::default();
    let mut query = app
        .world_mut()
        .query::<(&clicker::PlayerNo, &clicker::ClickCounter)>();
    for (player, counter) in query.iter(app.world()) {
        if **counter != 0 {
            out.per.insert(testing::Player(**player), **counter);
        }
    }
    out.global = **app.world().resource::<clicker::GlobalCounter>();
    out
}

// no test_usage necessary
