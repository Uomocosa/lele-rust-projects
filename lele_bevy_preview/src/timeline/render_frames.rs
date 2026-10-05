use std::path::PathBuf;

use crate::Error;
use crate::deliver;
use crate::preview;
use crate::scene;
use crate::scene::Scene;
use crate::timeline;

pub fn render_frames(
    scene: &Scene,
    config: &preview::Config,
) -> Result<Vec<(PathBuf, String)>, Error> {
    scene::check::check(scene)?;
    let Some(spec) = scene.timeline.as_ref() else {
        return Ok(Vec::new());
    };
    let scratch = timeline::scratch_dir::scratch_dir(scene, config);
    let _ = std::fs::remove_dir_all(&scratch);
    let mut app = preview::build::build(scene, config);
    preview::warmup::warmup(&mut app, config);
    (spec.apply)(app.world_mut());
    let target = preview::target_of::target_of(&mut app);
    let mut frames = Vec::new();
    for index in 0..spec.frames {
        app.update();
        let path = scratch.join(format!("frame-{:04}.png", index.saturating_add(1)));
        let image =
            preview::once::once(&mut app, target.clone(), &path, config.max_capture_frames)?;
        let pixels = image.data.as_deref().unwrap_or_default();
        preview::ensure_visual::ensure_visual(pixels, &path, config.min_distinct_colors)?;
        frames.push((path, deliver::hash_bytes::hash_bytes(pixels)));
    }
    Ok(frames)
}

// no test_usage necessary
