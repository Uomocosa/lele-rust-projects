use bevy::prelude::*;

use crate::preview;
use crate::scene::Scene;

pub fn build(scene: &Scene, config: &preview::Config) -> App {
    let mut app = App::new();
    (scene.build)(&mut app);
    let _ = preview::install::install(&mut app, config);
    app
}

// no test_usage necessary
