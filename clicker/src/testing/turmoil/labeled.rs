use bevy::prelude::App;

use crate::clicker;

#[must_use]
pub fn labeled(app: &mut App) -> usize {
    let mut query = app.world_mut().query::<&clicker::PlayerNo>();
    query.iter(app.world()).count()
}

// no test_usage necessary
