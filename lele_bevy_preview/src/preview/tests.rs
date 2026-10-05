use crate::preview;
use bevy::prelude::*;

use crate::scene;

// needed helper: fixture shared by the tests in this file
fn noop(_world: &mut World) {}
// needed helper: fixture shared by the tests in this file
fn noop_build(_app: &mut App) {}

// needed helper: fixture shared by the tests in this file
fn scene() -> scene::Scene {
    scene::Scene {
        name: String::from("demo"),
        build: noop_build,
        states: vec![
            crate::scene::basic::structs::State {
                label: String::from("one room"),
                apply: noop,
            },
            crate::scene::basic::structs::State {
                label: String::from("no rooms!"),
                apply: noop,
            },
        ],
        timeline: None,
    }
}

// needed helper: fixture shared by the tests in this file
fn fast(dir: &std::path::Path) -> preview::Config {
    preview::Config {
        out_dir: dir.to_path_buf(),
        warmup_frames: 5,
        max_capture_frames: 240,
        min_distinct_colors: 1,
        ..preview::Config::default()
    }
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage() {
    let config = preview::Config::default();
    assert_eq!(config.width, preview::basic::constants::DEFAULT_WIDTH);
    assert_eq!(config.height, preview::basic::constants::DEFAULT_HEIGHT);
    assert_eq!(
        config.warmup_frames,
        preview::basic::constants::DEFAULT_WARMUP_FRAMES
    );
    assert!(config.out_dir.ends_with("lele-bevy-preview"));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_install_produces_an_offscreen_target_and_a_camera() {
    let config = preview::Config::default();
    let mut app = App::new();
    let target = preview::install(&mut app, &config);
    assert_ne!(target, Handle::default());
    let world = app.world_mut();
    let mut cameras = world.query_filtered::<Entity, With<bevy::camera::RenderTarget>>();
    assert!(cameras.iter(world).count() > 0);
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_once_writes_a_non_empty_png_and_creates_parents() {
    let config = preview::Config::default();
    let mut app = App::new();
    let target = preview::install(&mut app, &config);
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("nested/deeper/frame.png");
    let image = preview::once(&mut app, target, &out, 240).unwrap();
    assert!(image.data.is_some_and(|data| !data.is_empty()));
    assert!(out.exists());
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_settled_returns_a_frame_once_two_readbacks_match() {
    let config = preview::Config::default();
    let mut app = App::new();
    let target = preview::install(&mut app, &config);
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("settled.png");
    let image = preview::settled(&mut app, &target, &out, 240, 1).unwrap();
    assert!(image.data.is_some_and(|data| !data.is_empty()));
    assert!(out.exists());
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_a_zero_frame_budget_reports_no_frame() {
    let config = preview::Config::default();
    let mut app = App::new();
    let target = preview::install(&mut app, &config);
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("missing.png");
    assert!(matches!(
        preview::once(&mut app, target, &out, 0),
        Err(crate::Error::NoFrame(0))
    ));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_render_states_writes_one_png_per_state_named_after_the_scene() {
    let dir = tempfile::tempdir().unwrap();
    let rendered = preview::render_states(&scene(), &fast(dir.path())).unwrap();
    assert_eq!(rendered.len(), 2);
    for (path, label, pixels) in &rendered {
        assert!(path.exists(), "{} missing", path.display());
        assert!(label.len() > 1);
        assert_eq!(pixels.len(), crate::deliver::basic::constants::HASH_CHARS);
    }
    assert!(rendered[0].0.to_string_lossy().contains("demo__one_room"));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_render_states_rejects_an_invalid_scene_before_rendering() {
    let dir = tempfile::tempdir().unwrap();
    let broken = scene::Scene {
        states: Vec::new(),
        ..scene()
    };
    assert!(preview::render_states(&broken, &fast(dir.path())).is_err());
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_build_warmup_and_target_of_are_usable_together() {
    let dir = tempfile::tempdir().unwrap();
    let config = fast(dir.path());
    let mut app = preview::build(&scene(), &config);
    preview::warmup(&mut app, &config);
    app.update();
    assert_ne!(preview::target_of(&mut app), Handle::default());
}
// no test_usage necessary
