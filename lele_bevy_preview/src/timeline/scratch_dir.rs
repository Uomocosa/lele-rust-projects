use std::path::PathBuf;

use crate::preview;
use crate::scene::Scene;

#[must_use]
pub fn scratch_dir(scene: &Scene, config: &preview::Config) -> PathBuf {
    config.out_dir.join(format!("{}_frames", scene.name))
}

// no test_usage necessary
