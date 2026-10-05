use crate::preview;
use bevy::prelude::*;

pub fn warmup(app: &mut App, config: &preview::Config) {
    for _ in 0..config.warmup_frames {
        app.update();
    }
}
// no test_usage necessary
