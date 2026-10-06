use crate::Error;
use crate::deliver;
use crate::preview;
use crate::report::Report;
use crate::scene::Scene;
use crate::timeline;

pub fn run(scene: &Scene, config: &preview::Config, crate_name: &str) -> Result<Report, Error> {
    let rendered = preview::render_states::render_states(scene, config)?;
    let previous = deliver::load_previous::load_previous(&config.out_dir);
    let captures =
        deliver::collect::collect(&scene.name, scene.kind, &rendered, previous.as_ref())?;
    let clip = timeline::build_clip::build_clip(scene, config)?;
    deliver::update_manifest::update_manifest(
        &config.out_dir,
        crate_name,
        &captures,
        clip.as_ref(),
    )?;
    deliver::send_changed::send_changed(scene, &captures, clip.as_ref())?;
    announce(&captures, clip.as_ref());
    Ok(Report { captures, clip })
}

// needed helper: one line per artifact so a human reviewing test output sees what changed
fn announce(captures: &[deliver::Capture], clip: Option<&deliver::Capture>) {
    for capture in captures {
        println!(
            "PREVIEW_ARTIFACT={} status={} label={}",
            capture.path.display(),
            label(capture.status),
            capture.artifact.label
        );
    }
    if let Some(clip) = clip {
        println!(
            "PREVIEW_ARTIFACT={} status={} label={}",
            clip.path.display(),
            label(clip.status),
            clip.artifact.label
        );
    }
    println!(
        "PREVIEW_MANIFEST={}",
        deliver::basic::constants::MANIFEST_FILE
    );
}

const fn label(status: deliver::Status) -> &'static str {
    match status {
        deliver::Status::FirstRun => "first-run",
        deliver::Status::New => "new",
        deliver::Status::Changed => "changed",
        deliver::Status::Same => "same",
    }
}
// no test_usage necessary
