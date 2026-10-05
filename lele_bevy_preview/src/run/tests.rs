use crate::run;
use std::path::PathBuf;

use bevy::ecs::world::World;
use bevy::prelude::App;

use crate::Error;
use crate::deliver;
use crate::preview;
use crate::scene::Scene;
use crate::scene::basic::structs::State;

// needed helper: fixture shared by the tests in this file
fn noop(_world: &mut World) {}
// needed helper: fixture shared by the tests in this file
fn noop_build(_app: &mut App) {}

// needed helper: fixture shared by the tests in this file
fn scene() -> Scene {
    Scene {
        name: String::from("sync_room_list"),
        build: noop_build,
        states: vec![State {
            label: String::from("two rooms"),
            apply: noop,
        }],
        timeline: None,
    }
}

// needed helper: fixture shared by the tests in this file
fn config(dir: &std::path::Path) -> preview::Config {
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
    let dir = tempfile::tempdir().unwrap();
    let report = run(&scene(), &config(dir.path()), "demo").unwrap();
    assert_eq!(report.captures.len(), 1);
    assert_eq!(report.captures[0].status, deliver::Status::FirstRun);
    assert!(report.clip.is_none());
    assert!(
        dir.path()
            .join(deliver::basic::constants::MANIFEST_FILE)
            .exists()
    );
    assert!(report.to_string().contains("two rooms"));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_second_identical_run_reports_same_and_sends_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let report = run(&scene(), &config(dir.path()), "demo").unwrap();
    let second = run(&scene(), &config(dir.path()), "demo").unwrap();
    let first_hash = &report.captures[0].artifact.pixel_hash;
    assert_eq!(
        &second.captures[0].artifact.pixel_hash, first_hash,
        "two identical runs must hash identically, otherwise the change gate is noise"
    );
    assert_eq!(second.captures[0].status, deliver::Status::Same);
    assert!(!crate::deliver::sendable::sendable(
        second.captures[0].status
    ));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_differing_pixels_report_changed() {
    let dir = tempfile::tempdir().unwrap();
    run(&scene(), &config(dir.path()), "demo").unwrap();
    let mut manifest = deliver::load_previous::load_previous(dir.path()).unwrap();
    for entry in manifest.artifacts.values_mut() {
        entry.pixel_hash = String::from("stale");
    }
    std::fs::write(
        dir.path().join("preview-manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    let third = run(&scene(), &config(dir.path()), "demo").unwrap();
    assert_eq!(third.captures[0].status, deliver::Status::Changed);
    assert!(crate::deliver::sendable::sendable(third.captures[0].status));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_manifest_accumulates_across_scenes() {
    let dir = tempfile::tempdir().unwrap();
    run(&scene(), &config(dir.path()), "demo").unwrap();
    let other = Scene {
        name: String::from("spawn_root"),
        ..scene()
    };
    run(&other, &config(dir.path()), "demo").unwrap();
    let merged = deliver::load_previous::load_previous(dir.path()).unwrap();
    assert_eq!(merged.artifacts.len(), 2);
    assert!(merged.artifacts.contains_key("sync_room_list|two rooms"));
    assert!(merged.artifacts.contains_key("spawn_root|two rooms"));
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_an_invalid_scene_is_reported_before_any_capture() {
    let dir = tempfile::tempdir().unwrap();
    let broken = Scene {
        states: Vec::new(),
        ..scene()
    };
    let result = run(&broken, &config(dir.path()), "demo");
    assert!(matches!(result, Err(Error::Scene(_))));
    assert!(
        !dir.path()
            .join(deliver::basic::constants::MANIFEST_FILE)
            .exists()
    );
}

#[test]
// needed helper: fixture shared by the tests in this file
fn test_usage_artifacts_are_named_after_the_scene_and_state() {
    let dir = tempfile::tempdir().unwrap();
    let report = run(&scene(), &config(dir.path()), "demo").unwrap();
    let path: PathBuf = report.captures[0].path.clone();
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    assert_eq!(name, "sync_room_list__two_rooms.png");
}
// no test_usage necessary
