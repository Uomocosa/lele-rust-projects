use std::path::PathBuf;

use crate::Error;
use crate::deliver;
use crate::deliver::basic::enums::Media;
use crate::deliver::basic::structs::{Artifact, Capture};
use crate::preview;
use crate::scene::Scene;
use crate::timeline;

pub fn build_clip(scene: &Scene, config: &preview::Config) -> Result<Option<Capture>, Error> {
    let Some(spec) = scene.timeline.as_ref() else {
        return Ok(None);
    };
    let rendered = timeline::render_frames::render_frames(scene, config)?;
    let (frames, hashes): (Vec<PathBuf>, Vec<String>) = rendered.into_iter().unzip();
    if hashes.windows(2).all(|pair| pair.first() == pair.get(1)) {
        return Err(Error::StaticClip {
            scene: scene.name.clone(),
            label: spec.label.clone(),
        });
    }
    let out = config.out_dir.join(format!(
        "{}__{}.mp4",
        scene.name,
        deliver::slug::slug(&spec.label)
    ));
    timeline::build::build(
        &frames,
        spec.fps,
        &out,
        &timeline::scratch_dir::scratch_dir(scene, config),
    )?;
    let artifact = Artifact {
        scene: scene.name.clone(),
        label: spec.label.clone(),
        fingerprint: deliver::fingerprint_of::fingerprint_of(
            &scene.name,
            &format!("clip:{}", spec.label),
        ),
        pixel_hash: clip_hash(spec.fps, &hashes),
        media: Media::Mp4,
        kind: scene.kind,
    };
    let previous = deliver::load_previous::load_previous(&config.out_dir);
    let status = deliver::state_status::state_status(previous.as_ref(), &artifact);
    Ok(Some(Capture {
        path: out,
        artifact,
        status,
    }))
}

// needed helper: content hash over the clip's source frames so a no-op re-encode stays Same
fn clip_hash(fps: u32, hashes: &[String]) -> String {
    let joined = format!("{fps}:{}", hashes.join(","));
    deliver::hash_bytes::hash_bytes(joined.as_bytes())
}

// no test_usage necessary
