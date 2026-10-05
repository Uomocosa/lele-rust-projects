use std::path::PathBuf;

use crate::Error;
use crate::deliver;
use crate::preview;
use crate::scene;
use crate::scene::Scene;

pub fn render_states(
    scene: &Scene,
    config: &preview::Config,
) -> Result<Vec<(PathBuf, String, String)>, Error> {
    scene::check::check(scene)?;
    let mut rendered = Vec::new();
    for state in &scene.states {
        let out = config.out_dir.join(format!(
            "{}__{}.png",
            scene.name,
            deliver::slug::slug(&state.label)
        ));
        let pixels = render_one(scene, config, state, &out)?;
        rendered.push((out, state.label.clone(), pixels));
    }
    Ok(rendered)
}

// needed helper: every state needs its own App, because a spawn from state N must not leak into N+1
fn render_one(
    scene: &Scene,
    config: &preview::Config,
    state: &scene::State,
    out: &std::path::Path,
) -> Result<String, Error> {
    let mut app = preview::build::build(scene, config);
    preview::warmup::warmup(&mut app, config);
    (state.apply)(app.world_mut());
    app.update();
    let target = preview::target_of::target_of(&mut app);
    let image = preview::settled::settled(
        &mut app,
        &target,
        out,
        config.max_capture_frames,
        config.min_distinct_colors,
    )?;
    Ok(deliver::hash_bytes::hash_bytes(
        image.data.as_deref().unwrap_or_default(),
    ))
}
// no test_usage necessary
