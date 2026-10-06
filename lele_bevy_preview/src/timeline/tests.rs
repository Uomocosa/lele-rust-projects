use crate::timeline;
use bevy::ecs::world::World;
use bevy::prelude::App;

use crate::preview;
use crate::scene::Scene;
use crate::scene::basic::structs::{State, Timeline};

// needed helper: fixture shared by the tests in this file
fn noop(_world: &mut World) {}
// needed helper: a visible change, because a clip whose frames never differ is rejected
fn appear(world: &mut World) {
    world.spawn((
        bevy::ui::Node {
            width: bevy::ui::Val::Px(200.0),
            height: bevy::ui::Val::Px(120.0),
            ..Default::default()
        },
        bevy::ui::BackgroundColor(bevy::color::Color::srgb(0.9, 0.2, 0.2)),
    ));
}
// needed helper: fixture shared by the tests in this file
fn noop_build(_app: &mut App) {}

// needed helper: fixture shared by the tests in this file
fn scene(spec: Option<Timeline>) -> Scene {
    Scene {
        name: String::from("press"),
        kind: crate::scene::Kind::System,
        build: noop_build,
        states: vec![State {
            label: String::from("rest"),
            apply: noop,
        }],
        timeline: spec,
    }
}

// needed helper: fixture shared by the tests in this file
fn spec(frames: u32) -> Timeline {
    Timeline {
        label: String::from("hover_then_press"),
        frames,
        fps: 30,
        apply: appear,
    }
}

// needed helper: fixture shared by the tests in this file
fn fast(dir: &std::path::Path) -> preview::Config {
    preview::Config {
        out_dir: dir.to_path_buf(),
        warmup_frames: 5,
        lead_in_frames: 2,
        max_capture_frames: 240,
        min_distinct_colors: 1,
        ..preview::Config::default()
    }
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage() {
    let dir = tempfile::tempdir().unwrap();
    let clip = timeline::build_clip::build_clip(&scene(Some(spec(3))), &fast(dir.path()))
        .unwrap()
        .expect("a declared timeline must produce a clip");
    assert!(clip.path.exists(), "{} missing", clip.path.display());
    assert!(clip.path.extension().is_some_and(|ext| ext == "mp4"));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_renders_the_lead_in_then_one_png_per_declared_frame() {
    let dir = tempfile::tempdir().unwrap();
    let frames =
        timeline::render_frames::render_frames(&scene(Some(spec(3))), &fast(dir.path())).unwrap();
    assert_eq!(frames.len(), 5, "2 lead-in frames plus the 3 declared ones");
    for frame in &frames {
        assert!(frame.0.exists(), "{} missing", frame.0.display());
    }
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_a_scene_without_a_timeline_produces_no_frames() {
    let dir = tempfile::tempdir().unwrap();
    let config = fast(dir.path());
    assert_eq!(
        timeline::render_frames::render_frames(&scene(None), &config)
            .unwrap()
            .len(),
        0
    );
    assert!(
        timeline::build_clip::build_clip(&scene(None), &config)
            .unwrap()
            .is_none()
    );
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_two_runs_produce_identical_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let config = fast(dir.path());
    let one = timeline::render_frames::render_frames(&scene(Some(spec(2))), &config).unwrap();
    let first = std::fs::read(&one[1].0).unwrap();
    let two = timeline::render_frames::render_frames(&scene(Some(spec(2))), &config).unwrap();
    let second = std::fs::read(&two[1].0).unwrap();
    assert_eq!(
        first, second,
        "fixed-step rendering must be reproducible frame to frame"
    );
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_empty_frame_list_is_an_encoding_error() {
    let dir = tempfile::tempdir().unwrap();
    let scratch = dir.path().join("frames");
    let out = dir.path().join("clip.mp4");
    let empty: Vec<std::path::PathBuf> = Vec::new();
    assert!(matches!(
        timeline::build::build(&empty, 30, &out, &scratch),
        Err(crate::Error::Ffmpeg { .. })
    ));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_a_missing_ffmpeg_binary_is_reported_not_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let scratch = dir.path().join("frames");
    std::fs::create_dir_all(&scratch).unwrap();
    let frame = scratch.join("frame-0001.png");
    std::fs::write(&frame, b"not really a png").unwrap();
    unsafe { std::env::set_var("LELE_FFMPEG", "/nonexistent/ffmpeg-binary") };
    let out = dir.path().join("clip.mp4");
    let result = timeline::build::build(&[frame], 30, &out, &scratch);
    unsafe { std::env::remove_var("LELE_FFMPEG") };
    assert!(matches!(result, Err(crate::Error::Ffmpeg { .. })));
}
// no test_usage necessary

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_a_clip_whose_frames_never_differ_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let flat = Timeline {
        apply: noop,
        ..spec(3)
    };
    let result = timeline::build_clip::build_clip(&scene(Some(flat)), &fast(dir.path()));
    assert!(matches!(result, Err(crate::Error::StaticClip { .. })));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_the_baseline_precedes_the_change() {
    let dir = tempfile::tempdir().unwrap();
    let frames =
        timeline::render_frames::render_frames(&scene(Some(spec(3))), &fast(dir.path())).unwrap();
    assert_eq!(frames[0].1, frames[1].1, "lead-in frames show the baseline");
    assert_ne!(
        frames[1].1, frames[4].1,
        "the change shows up after the lead-in"
    );
}
